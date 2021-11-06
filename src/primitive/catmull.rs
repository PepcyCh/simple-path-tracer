use std::{collections::HashSet, sync::Arc};

use pep_mesh::{halfedge, io::ply};

use crate::{
    core::{
        bbox::Bbox,
        intersection::Intersection,
        primitive::{Aggregate, Primitive},
        ray::Ray,
        sampler::Sampler,
        scene::Scene,
    },
    loader::{self, JsonObject, LoadableSceneObject},
    primitive::BvhAccel,
};

use super::CubicBezier;

pub struct VData {
    pos: glam::Vec3A,
    new_pos: Option<glam::Vec3A>,
}

impl Default for VData {
    fn default() -> Self {
        Self {
            pos: glam::Vec3A::new(0.0, 0.0, 0.0),
            new_pos: None,
        }
    }
}

impl From<ply::PropertyMap> for VData {
    fn from(props: ply::PropertyMap) -> Self {
        let x = props.map.get("x").map_or(0.0, |prop| match prop {
            ply::Property::F32(val) => *val,
            ply::Property::F64(val) => *val as f32,
            _ => 0.0,
        });
        let y = props.map.get("y").map_or(0.0, |prop| match prop {
            ply::Property::F32(val) => *val,
            ply::Property::F64(val) => *val as f32,
            _ => 0.0,
        });
        let z = props.map.get("z").map_or(0.0, |prop| match prop {
            ply::Property::F32(val) => *val,
            ply::Property::F64(val) => *val as f32,
            _ => 0.0,
        });
        let pos = glam::Vec3A::new(x, y, z);

        Self {
            pos,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct EData {
    new_pos: Option<glam::Vec3A>,
    new_vert: Option<halfedge::VertexRef>,
    sharpness: f32,
}

impl From<ply::PropertyMap> for EData {
    fn from(props: ply::PropertyMap) -> Self {
        let sharpness = props.map.get("sharpness").map_or(0.0, |prop| match prop {
            ply::Property::F32(val) => *val,
            ply::Property::F64(val) => *val as f32,
            _ => 0.0,
        });
        Self {
            sharpness,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct FData {
    new_pos: Option<glam::Vec3A>,
    is_regular: bool,
}

impl From<ply::PropertyMap> for FData {
    fn from(_: ply::PropertyMap) -> Self {
        Self::default()
    }
}

type Mesh = halfedge::HalfEdgeMesh<VData, EData, FData>;

pub struct CatmullClark {
    bbox: Bbox,
    patches: Box<dyn Aggregate>,
}

impl CatmullClark {
    pub fn new(mut mesh: Mesh, fas_times: u32) -> Self {
        let patches = feature_adaptive_subdivision(&mut mesh, fas_times);
        let bbox = patches.bbox();

        Self { bbox, patches }
    }
}

impl Primitive for CatmullClark {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        self.patches.intersect_test(ray, t_max)
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        self.patches.intersect(ray, inter)
    }

    fn bbox(&self) -> Bbox {
        self.bbox
    }

    fn sample<'a>(&'a self, _sampler: &mut dyn Sampler) -> (Intersection<'a>, f32) {
        unimplemented!("<CatmullClark as Primitive>::sample() not supported yet")
    }

    fn pdf(&self, _inter: &Intersection<'_>) -> f32 {
        unimplemented!("<CatmullClark as Primitive>::pdf() not supported yet")
    }
}

fn feature_adaptive_subdivision(mesh: &mut Mesh, max_iter_times: u32) -> Box<dyn Aggregate> {
    let mut process_faces = mesh.faces().collect::<Vec<_>>();

    let mut patches = vec![];

    for _ in 0..max_iter_times {
        let mut irregular_faces = vec![];

        // find irregular faces
        for face in &process_faces {
            if face.is_boundary(mesh) {
                continue;
            }

            let is_regular = check_if_regular(face, &mesh);
            face.data_mut(mesh).is_regular = is_regular;
            if !is_regular {
                irregular_faces.push(*face);
            } else {
                patches.push(get_bezier_patch(face, mesh));
            }
        }

        // collect faces that need to be subdivided
        let mut to_be_subdivided = HashSet::new();
        for face in irregular_faces {
            let mut he = face.halfedge(mesh);
            loop {
                let mut vhe = he;
                loop {
                    let f = vhe.face(mesh);
                    if !f.is_boundary(mesh) {
                        to_be_subdivided.insert(f);
                    }
                    vhe = vhe.twin(mesh).next(mesh);
                    if vhe == he {
                        break;
                    }
                }
                he = he.next(mesh);
                if he == face.halfedge(mesh) {
                    break;
                }
            }
        }
        let to_be_subdivided = to_be_subdivided.into_iter().collect::<Vec<_>>();

        // calc position of face point
        for face in &to_be_subdivided {
            let mut count = 0.0;
            let mut sum = glam::Vec3A::new(0.0, 0.0, 0.0);
            let mut he = face.halfedge(mesh);
            loop {
                count += 1.0;
                sum += he.vertex(mesh).data(mesh).pos;
                he = he.next(mesh);
                if he == face.halfedge(mesh) {
                    break;
                }
            }
            face.data_mut(mesh).new_pos = Some(sum / count);
        }
        // calc position of edge point
        for face in &to_be_subdivided {
            let mut he = face.halfedge(mesh);
            loop {
                if he.data(mesh).new_pos.is_none() {
                    let twin = he.twin(mesh);

                    let pos_v1 = he.vertex(mesh).data(mesh).pos;
                    let pos_v2 = twin.vertex(mesh).data(mesh).pos;

                    let sharpness = he.data(mesh).sharpness;

                    let pos_crease = (pos_v1 + pos_v2) * 0.5;
                    let pos_smooth = {
                        let pos_f1 = he.face(mesh).data(mesh).new_pos;
                        let pos_f2 = twin.face(mesh).data(mesh).new_pos;
                        if pos_f1.is_some() && pos_f2.is_some() {
                            let pos_f1 = pos_f1.unwrap();
                            let pos_f2 = pos_f2.unwrap();
                            (pos_v1 + pos_v2 + pos_f1 + pos_f2) * 0.25
                        } else {
                            (pos_v1 + pos_v2) * 0.5
                        }
                    };

                    if he.on_boundary(mesh) || twin.on_boundary(mesh) || sharpness >= 1.0 {
                        he.data_mut(mesh).new_pos = Some(pos_crease);
                    } else if sharpness > 0.0 {
                        he.data_mut(mesh).new_pos =
                            Some(pos_crease * sharpness + pos_smooth * (1.0 - sharpness));
                    } else {
                        he.data_mut(mesh).new_pos = Some(pos_smooth);
                    }
                }
                he = he.next(mesh);
                if he == face.halfedge(mesh) {
                    break;
                }
            }
        }
        // calc position of vertex point
        for face in &to_be_subdivided {
            let mut he = face.halfedge(mesh);
            loop {
                let v = he.vertex(mesh);
                if v.data(mesh).new_pos.is_none() {
                    let mut crease_he_pos1 = None;
                    let mut crease_he_pos2 = None;
                    let num_creases = {
                        let mut he = v.halfedge(mesh);
                        let mut num_creases = 0;
                        loop {
                            let twin = he.twin(mesh);
                            if he.data(mesh).sharpness > 0.0
                                || he.on_boundary(mesh)
                                || twin.on_boundary(mesh)
                            {
                                num_creases += 1;
                                if crease_he_pos1.is_none() {
                                    crease_he_pos1 = Some(twin.vertex(mesh).data(mesh).pos);
                                } else if crease_he_pos2.is_none() {
                                    crease_he_pos2 = Some(twin.vertex(mesh).data(mesh).pos);
                                }
                            }
                            he = twin.next(mesh);
                            if he == v.halfedge(mesh) {
                                break;
                            }
                        }
                        num_creases
                    };

                    let pos_v = v.data(mesh).pos;

                    if num_creases > 2 {
                        v.data_mut(mesh).new_pos = Some(pos_v);
                    } else if num_creases == 2 {
                        let pos_crease = {
                            assert!(crease_he_pos1.is_some() && crease_he_pos2.is_some());
                            let pos1 = crease_he_pos1.unwrap();
                            let pos2 = crease_he_pos2.unwrap();
                            0.75 * pos_v + 0.125 * pos1 + 0.125 * pos2
                        };
                        v.data_mut(mesh).new_pos = Some(pos_crease);
                    } else {
                        let pos_smooth = {
                            let mut n = 0.0;
                            let mut sum = glam::Vec3A::new(0.0, 0.0, 0.0);
                            let mut vhe = he;
                            loop {
                                let twin = vhe.twin(mesh);
                                n += 1.0;
                                sum += twin.vertex(mesh).data(mesh).pos;
                                if let Some(pos_f) = vhe.face(mesh).data(mesh).new_pos {
                                    sum += pos_f;
                                } else {
                                    sum += pos_v;
                                }
                                vhe = twin.next(mesh);
                                if vhe == he {
                                    break;
                                }
                            }
                            let n_inv = 1.0 / n;
                            ((n - 2.0) * pos_v + sum * n_inv) * n_inv
                        };
                        v.data_mut(mesh).new_pos = Some(pos_smooth);
                    }
                }
                he = he.next(mesh);
                if he == face.halfedge(mesh) {
                    break;
                }
            }
        }
        // split edges
        let mut halfedges = vec![];
        for face in &to_be_subdivided {
            let mut he = face.halfedge(mesh);
            loop {
                if he.data(mesh).new_vert.is_none() {
                    he.data_mut(mesh).new_vert = Some(mesh.create_vertex(VData {
                        pos: he.data(mesh).new_pos.unwrap(),
                        new_pos: None,
                    }));
                    halfedges.push(he);
                }
                he = he.next(mesh);
                if he == face.halfedge(mesh) {
                    break;
                }
            }
        }
        for he in halfedges {
            let ev = he.data(mesh).new_vert.unwrap();

            let new_sharpness = (he.data(mesh).sharpness - 1.0).max(0.0);
            *he.data_mut(mesh) = EData {
                sharpness: new_sharpness,
                ..Default::default()
            };

            let new_edge = mesh.create_edge(
                &he.vertex(mesh),
                &ev,
                EData {
                    sharpness: new_sharpness,
                    ..Default::default()
                },
            );
            let twin = he.twin(mesh);

            new_edge.0.set_next(mesh, &he);
            he.last(mesh).set_next(mesh, &new_edge.0);
            he.vertex(mesh).set_halfedge(mesh, &new_edge.0);
            he.set_vertex(mesh, &ev);
            new_edge.0.set_face(mesh, &he.face(mesh));
            he.face(mesh).set_halfedge(mesh, &new_edge.0);

            new_edge.1.set_next(mesh, &twin.next(mesh));
            twin.set_next(mesh, &new_edge.1);
            new_edge.1.set_face(mesh, &twin.face(mesh));
            twin.face(mesh).set_halfedge(mesh, &twin);
            ev.set_halfedge(mesh, &he);
        }
        // update topology and process_faces
        process_faces.clear();
        for face in &to_be_subdivided {
            let fv = mesh.create_vertex(VData {
                pos: face.data(mesh).new_pos.unwrap(),
                new_pos: None,
            });
            let is_regular = face.data(mesh).is_regular;
            *face.data_mut(mesh) = FData::default();
            let mut new_edges = vec![];
            let mut new_faces = vec![];
            let mut count = 0;

            let mut he = face.halfedge(mesh);
            loop {
                let he_last = he;
                he = he.next(mesh);
                let ev = he.vertex(mesh);
                let he_next = he;

                let new_edge = mesh.create_edge(&ev, &fv, EData::default());
                fv.set_halfedge(mesh, &new_edge.1);
                new_edge.1.set_next(mesh, &he_next);
                he_last.set_next(mesh, &new_edge.0);

                new_edges.push(new_edge.0);
                new_edges.push(new_edge.1);

                if count == 0 {
                    new_faces.push(*face);
                } else {
                    let new_face = mesh.create_face(FData::default(), face.is_boundary(mesh));
                    new_faces.push(new_face);
                }
                count += 1;

                let vert = he_last.vertex(mesh);
                if let Some(v_pos) = vert.data(mesh).new_pos {
                    vert.data_mut(mesh).pos = v_pos;
                    vert.data_mut(mesh).new_pos = None;
                }

                he = he.next(mesh);
                if he == face.halfedge(mesh) {
                    break;
                }
            }

            for i in 0..count {
                let j = if i == 0 { 2 * count - 1 } else { 2 * i - 1 };
                let edge_j = new_edges[j];
                new_edges[2 * i].set_next(mesh, &edge_j);
                new_faces[i].set_halfedge(mesh, &edge_j);
                let mut he = new_edges[2 * i];
                loop {
                    he.set_face(mesh, &new_faces[i]);
                    he = he.next(mesh);
                    if he == new_edges[2 * i] {
                        break;
                    }
                }
            }

            if !is_regular {
                process_faces.append(&mut new_faces);
            }
        }
    }

    for face in &process_faces {
        if face.is_boundary(mesh) {
            continue;
        }

        let is_regular = check_if_regular(face, &mesh);
        if !is_regular {
            patches.push(get_gregory_patch(face, mesh));
        } else {
            patches.push(get_bezier_patch(face, mesh));
        }
    }

    Box::new(BvhAccel::new(patches, 4, 16))
}

fn check_if_regular(face: &halfedge::FaceRef, mesh: &Mesh) -> bool {
    let face_deg = face.degree(mesh);

    let mut he = face.halfedge(mesh);
    let mut is_regular = face_deg == 4;
    while is_regular {
        let v = he.vertex(mesh);
        let mut vert_deg = v.degree(mesh);
        if v.on_boundary(mesh) {
            vert_deg += 1;
        }
        if vert_deg != 4 || he.data(mesh).sharpness > 0.0 {
            is_regular = false;
        } else {
            he = he.next(mesh);
            if he == face.halfedge(mesh) {
                break;
            }
        }
    }

    is_regular
}

fn get_bezier_patch(face: &halfedge::FaceRef, mesh: &Mesh) -> Arc<dyn Primitive> {
    let mut control_points = [[glam::Vec3A::new(0.0, 0.0, 0.0); 4]; 4];

    let order = [
        [(1, 1), (0, 1), (1, 0), (0, 0)],
        [(1, 2), (1, 3), (0, 2), (0, 3)],
        [(2, 2), (3, 2), (2, 3), (3, 3)],
        [(2, 1), (2, 0), (3, 1), (3, 0)],
    ];

    let mut he = face.halfedge(mesh);
    for order in &order {
        let last = he;
        he = he.next(mesh);

        let p0 = order[0];
        let p1 = order[1];
        let p2 = order[2];
        let p3 = order[3];

        control_points[p0.0][p0.1] = he.vertex(mesh).data(mesh).pos;
        if he.twin(mesh).on_boundary(mesh) {
            let pos_temp = last.vertex(mesh).data(mesh).pos;
            control_points[p1.0][p1.1] =
                control_points[p0.0][p0.1] + (control_points[p0.0][p0.1] - pos_temp);
            let he2 = he.twin(mesh).next(mesh).twin(mesh);
            control_points[p2.0][p2.1] = he2.vertex(mesh).data(mesh).pos;
            let pos_temp = he2.last(mesh).vertex(mesh).data(mesh).pos;
            control_points[p3.0][p3.1] =
                control_points[p2.0][p2.1] + (control_points[p2.0][p2.1] - pos_temp);
        } else if last.twin(mesh).on_boundary(mesh) {
            let he2 = he.twin(mesh).next(mesh).twin(mesh);
            control_points[p1.0][p1.1] = he2.vertex(mesh).data(mesh).pos;
            let pos_temp = he.twin(mesh).vertex(mesh).data(mesh).pos;
            control_points[p2.0][p2.1] =
                control_points[p0.0][p0.1] + (control_points[p0.0][p0.1] - pos_temp);
            let pos_temp = he2
                .twin(mesh)
                .next(mesh)
                .twin(mesh)
                .vertex(mesh)
                .data(mesh)
                .pos;
            control_points[p3.0][p3.1] =
                control_points[p1.0][p1.1] + (control_points[p1.0][p1.1] - pos_temp);
        } else {
            let he2 = he.twin(mesh).next(mesh).twin(mesh);
            control_points[p1.0][p1.1] = he2.vertex(mesh).data(mesh).pos;
            let he2 = he2.next(mesh).next(mesh);
            control_points[p2.0][p2.1] = he2.vertex(mesh).data(mesh).pos;
            let he2 = he2.next(mesh);
            control_points[p3.0][p3.1] = he2.vertex(mesh).data(mesh).pos;
        }
    }

    let trans_mat = [
        [1.0 / 6.0, 4.0 / 6.0, 1.0 / 6.0, 0.0],
        [0.0, 4.0 / 6.0, 2.0 / 6.0, 0.0],
        [0.0, 2.0 / 6.0, 4.0 / 6.0, 0.0],
        [0.0, 1.0 / 6.0, 4.0 / 6.0, 1.0 / 6.0],
    ];

    let mut control_points_temp = [[glam::Vec3A::new(0.0, 0.0, 0.0); 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                control_points_temp[i][j] += control_points[i][k] * trans_mat[j][k];
            }
        }
    }

    for i in 0..4 {
        for j in 0..4 {
            control_points[j][i] = glam::Vec3A::new(0.0, 0.0, 0.0);
            for k in 0..4 {
                control_points[j][i] += control_points_temp[k][i] * trans_mat[j][k];
            }
        }
    }

    Arc::new(CubicBezier::new(control_points))
}

fn get_gregory_patch(face: &halfedge::FaceRef, mesh: &Mesh) -> Arc<dyn Primitive> {
    let mut control_points = [[glam::Vec3A::new(0.0, 0.0, 0.0); 4]; 4];

    let he = face.halfedge(mesh);
    let vert = he.vertex(mesh);
    let (edge_points0, face_points0) = get_edge_points_and_face_points(&vert, face, mesh);
    let pos0 = calc_vertex_control_point(vert.data(mesh).pos, &edge_points0, &face_points0);
    let (e0_pos, e0_neg) = calc_edge_control_points(pos0, &edge_points0, &face_points0);
    control_points[0][0] = pos0;
    control_points[0][1] = e0_pos;
    control_points[1][0] = e0_neg;

    let he = he.next(mesh);
    let vert = he.vertex(mesh);
    let (edge_points1, face_points1) = get_edge_points_and_face_points(&vert, face, mesh);
    let pos1 = calc_vertex_control_point(vert.data(mesh).pos, &edge_points1, &face_points1);
    let (e1_pos, e1_neg) = calc_edge_control_points(pos1, &edge_points1, &face_points1);
    control_points[0][3] = pos1;
    control_points[1][3] = e1_pos;
    control_points[0][2] = e1_neg;

    let he = he.next(mesh);
    let vert = he.vertex(mesh);
    let (edge_points2, face_points2) = get_edge_points_and_face_points(&vert, face, mesh);
    let pos2 = calc_vertex_control_point(vert.data(mesh).pos, &edge_points2, &face_points2);
    let (e2_pos, e2_neg) = calc_edge_control_points(pos2, &edge_points2, &face_points2);
    control_points[3][3] = pos2;
    control_points[3][2] = e2_pos;
    control_points[2][3] = e2_neg;

    let he = he.next(mesh);
    let vert = he.vertex(mesh);
    let (edge_points3, face_points3) = get_edge_points_and_face_points(&vert, face, mesh);
    let pos3 = calc_vertex_control_point(vert.data(mesh).pos, &edge_points3, &face_points3);
    let (e3_pos, e3_neg) = calc_edge_control_points(pos3, &edge_points3, &face_points3);
    control_points[3][0] = pos3;
    control_points[2][0] = e3_pos;
    control_points[3][1] = e3_neg;

    let n0 = edge_points0.len() as f32;
    let n1 = edge_points1.len() as f32;
    let n2 = edge_points2.len() as f32;
    let n3 = edge_points3.len() as f32;

    let f0_pos =
        calc_face_control_points_pos(pos0, e0_pos, e1_neg, &edge_points0, &face_points0, n0, n1);
    let f0_neg =
        calc_face_control_points_neg(pos0, e0_neg, e3_pos, &edge_points0, &face_points0, n0, n3);
    control_points[1][1] = (f0_pos + f0_neg) * 0.5;

    let f1_pos =
        calc_face_control_points_pos(pos1, e1_pos, e2_neg, &edge_points1, &face_points1, n1, n2);
    let f1_neg =
        calc_face_control_points_neg(pos1, e1_neg, e0_pos, &edge_points1, &face_points1, n1, n0);
    control_points[1][2] = (f1_pos + f1_neg) * 0.5;

    let f2_pos =
        calc_face_control_points_pos(pos2, e2_pos, e3_neg, &edge_points2, &face_points2, n2, n3);
    let f2_neg =
        calc_face_control_points_neg(pos2, e2_neg, e1_pos, &edge_points2, &face_points2, n2, n1);
    control_points[2][2] = (f2_pos + f2_neg) * 0.5;

    let f3_pos =
        calc_face_control_points_pos(pos3, e3_pos, e0_neg, &edge_points3, &face_points3, n3, n0);
    let f3_neg =
        calc_face_control_points_neg(pos3, e3_neg, e2_pos, &edge_points3, &face_points3, n3, n2);
    control_points[2][1] = (f3_pos + f3_neg) * 0.5;

    Arc::new(CubicBezier::new(control_points))
}

fn get_edge_points_and_face_points(
    vert: &halfedge::VertexRef,
    face: &halfedge::FaceRef,
    mesh: &Mesh,
) -> (Vec<glam::Vec3A>, Vec<glam::Vec3A>) {
    let mut edge_points = vec![];
    let mut face_points = vec![];

    let mut he = vert.halfedge(mesh);
    while he.face(mesh) != *face {
        he = he.twin(mesh).next(mesh);
    }
    he = he.twin(mesh).next(mesh);
    let start_he = he;

    let pos_v = vert.data(mesh).pos;
    loop {
        let twin = he.twin(mesh);
        let pos_e = (twin.vertex(mesh).data(mesh).pos + pos_v) * 0.5;
        edge_points.push(pos_e);

        let mut pos_f = glam::Vec3A::new(0.0, 0.0, 0.0);
        let mut fhe = he;
        let mut count_f = 0.0;
        loop {
            pos_f += fhe.vertex(mesh).data(mesh).pos;
            count_f += 1.0;
            fhe = fhe.next(mesh);
            if fhe == he {
                break;
            }
        }
        let pos_f = pos_f / count_f;
        face_points.push(pos_f);

        he = twin.next(mesh);
        if he == start_he {
            break;
        }
    }

    edge_points.reverse();
    face_points.reverse();

    (edge_points, face_points)
}

fn calc_vertex_control_point(
    pos_v: glam::Vec3A,
    edge_points: &[glam::Vec3A],
    face_points: &[glam::Vec3A],
) -> glam::Vec3A {
    let mut sum_ef = glam::Vec3A::new(0.0, 0.0, 0.0);
    for pos_e in edge_points {
        sum_ef += *pos_e;
    }
    for pos_f in face_points {
        sum_ef += *pos_f;
    }

    let n = edge_points.len() as f32;
    let n_inv = 1.0 / n;
    let n5_inv = 1.0 / (n + 5.0);

    let pos = (n - 3.0) * n5_inv * pos_v + 4.0 * n_inv * n5_inv * sum_ef;
    pos
}

fn calc_edge_control_points(
    pos_v: glam::Vec3A,
    edge_points: &[glam::Vec3A],
    face_points: &[glam::Vec3A],
) -> (glam::Vec3A, glam::Vec3A) {
    let n = edge_points.len() as f32;
    let n_inv = 1.0 / n;

    let frac_pi_n = std::f32::consts::PI * n_inv;
    let frac_pi_n_cos = frac_pi_n.cos();
    let frac_2pi_n = 2.0 * std::f32::consts::PI * n_inv;

    let sigma = 1.0 / (4.0 + frac_pi_n_cos * frac_pi_n_cos).sqrt();
    let temp = frac_2pi_n.cos();
    let lambda = (5.0 + temp + frac_pi_n_cos * (18.0 + 2.0 * temp).sqrt()) / 24.0;

    let mut tangent = glam::Vec3A::ZERO;
    let mut bitangent = glam::Vec3A::ZERO;
    let ka_common = 1.0 - sigma * frac_pi_n_cos;
    let kb_common = 2.0 * sigma;
    for i in 0..edge_points.len() {
        let ti = i as f32;
        let ka = ka_common * (frac_2pi_n * ti).cos();
        let kb = kb_common * (frac_2pi_n * ti + frac_pi_n).cos();
        tangent += ka * edge_points[i] + kb * face_points[i];

        let bi = ti - 1.0;
        let ka = ka_common * (frac_2pi_n * bi).cos();
        let kb = kb_common * (frac_2pi_n * bi + frac_pi_n).cos();
        bitangent += ka * edge_points[i] + kb * face_points[i];
    }
    let tangent = tangent * 2.0 * n_inv;
    let bitangent = bitangent * 2.0 * n_inv;

    let e_pos = pos_v + lambda * tangent;
    let e_neg = pos_v + lambda * bitangent;

    (e_pos, e_neg)
}

fn calc_face_control_points_pos(
    pos0: glam::Vec3A,
    e0_pos: glam::Vec3A,
    e1_neg: glam::Vec3A,
    edge_points0: &[glam::Vec3A],
    face_points0: &[glam::Vec3A],
    n0: f32,
    n1: f32,
) -> glam::Vec3A {
    let r = (edge_points0[edge_points0.len() - 1] - edge_points0[1]) / 3.0
        + 2.0 * (face_points0[0] - face_points0[face_points0.len() - 1]) / 3.0;
    let c0 = (2.0 * std::f32::consts::PI / n0).cos();
    let c1 = (2.0 * std::f32::consts::PI / n1).cos();
    (c1 * pos0 + (3.0 - 2.0 * c0 - c1) * e0_pos + 2.0 * c0 * e1_neg + r) / 3.0
}

fn calc_face_control_points_neg(
    pos0: glam::Vec3A,
    e0_neg: glam::Vec3A,
    e3_pos: glam::Vec3A,
    edge_points0: &[glam::Vec3A],
    face_points0: &[glam::Vec3A],
    n0: f32,
    n3: f32,
) -> glam::Vec3A {
    let r =
        (edge_points0[0] - edge_points0[2]) / 3.0 + 2.0 * (face_points0[0] - face_points0[1]) / 3.0;
    let c0 = (2.0 * std::f32::consts::PI / n0).cos();
    let c1 = (2.0 * std::f32::consts::PI / n3).cos();
    (c1 * pos0 + (3.0 - 2.0 * c0 - c1) * e0_neg + 2.0 * c0 * e3_pos + r) / 3.0
}

impl LoadableSceneObject for CatmullClark {
    fn load(
        scene: &mut Scene,
        path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "primitive-catmull_clark", "name")?;
        let env = format!("primitive-catmull_clark({})", name);
        if scene.primitives.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let file = loader::get_str_field(json_value, &env, "ply_file")?;
        let mesh = ply::load_to_halfedge(path.with_file_name(file))?;
        let fas_times = loader::get_int_field_or(json_value, &env, "fas_times", 4)?;

        let catmull = CatmullClark::new(mesh, fas_times);
        scene.primitives.insert(name.to_owned(), Arc::new(catmull));

        Ok(())
    }
}
