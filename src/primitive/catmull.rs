use std::{collections::HashSet, sync::Arc};

use cgmath::{ElementWise, EuclideanSpace, Point3, Vector3, Zero};
use pep_mesh::{halfedge, io::ply};

use crate::{core::{bbox::Bbox, intersection::Intersection, material::Material, medium::Medium, primitive::{Aggregate, Primitive}, ray::Ray}, primitive::BvhAccel};

use super::CubicBezier;

pub struct VData {
    pos: Point3<f32>,
    new_pos: Option<Point3<f32>>,
}

impl Default for VData {
    fn default() -> Self {
        Self {
            pos: Point3::new(0.0, 0.0, 0.0),
            new_pos: None,
        }
    }
}

impl From<ply::PropertyMap> for VData {
    fn from(props: ply::PropertyMap) -> Self {
        let x = props.map.get("x").map_or(0.0, |prop| {
            match prop {
                ply::Property::F32(val) => *val,
                ply::Property::F64(val) => *val as f32,
                _ => 0.0,
            }
        });
        let y = props.map.get("y").map_or(0.0, |prop| {
            match prop {
                ply::Property::F32(val) => *val,
                ply::Property::F64(val) => *val as f32,
                _ => 0.0,
            }
        });
        let z = props.map.get("z").map_or(0.0, |prop| {
            match prop {
                ply::Property::F32(val) => *val,
                ply::Property::F64(val) => *val as f32,
                _ => 0.0,
            }
        });
        let pos = Point3::new(x, y, z);

        Self {
            pos,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct EData {
    new_pos: Option<Point3<f32>>,
    new_vert: Option<halfedge::VertexRef>,
}

impl From<ply::PropertyMap> for EData {
    fn from(_: ply::PropertyMap) -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub struct FData {
    new_pos: Option<Point3<f32>>,
    is_regular: bool,
}

impl From<ply::PropertyMap> for FData {
    fn from(_: ply::PropertyMap) -> Self {
        Self::default()
    }
}

type Mesh = halfedge::HalfEdgeMesh<VData, EData, FData>;

pub struct CatmullClark {
    material: Arc<dyn Material>,
    bbox: Bbox,
    patches: Box<dyn Aggregate>,
}

impl CatmullClark {
    pub fn new(mut mesh: Mesh, material: Arc<dyn Material>) -> Self {
        let patches = feature_adaptive_subdivision(&mut mesh, 4, &material);
        let bbox = patches.bbox();

        Self {
            material,
            bbox,
            patches,
        }
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

    fn material(&self) -> Option<&dyn Material> {
        Some(&*self.material)
    }

    fn inside_medium(&self) -> Option<&dyn Medium> {
        None
    }
}

fn feature_adaptive_subdivision(
    mesh: &mut Mesh,
    max_iter_times: u32,
    material: &Arc<dyn Material>,
) -> Box<dyn Aggregate> {
    let mut process_faces = mesh.faces().collect::<Vec<_>>();

    let mut patches = vec![];

    for _ in 0..max_iter_times {
        let mut irregular_faces = vec![];

        // find irregular faces
        for face in &process_faces {
            if face.is_boundary(mesh) {
                continue;
            }

            let face_deg = face.degree(mesh);

            let mut he = face.halfedge(mesh);
            let mut is_regular = face_deg == 4;
            while is_regular {
                if he.vertex(mesh).degree(mesh) != 4 {
                    is_regular = false;
                } else {
                    he = he.next(mesh);
                    if he == face.halfedge(mesh) {
                        break;
                    }
                }
            }

            face.data_mut(mesh).is_regular = is_regular;
            if !is_regular {
                irregular_faces.push(*face);
            } else {
                patches.push(get_bezier_patch(face, mesh, material));
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
            let mut sum = Point3::new(0.0, 0.0, 0.0);
            let mut he = face.halfedge(mesh);
            loop {
                count += 1.0;
                sum.add_assign_element_wise(he.vertex(mesh).data(mesh).pos);
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

                    if he.on_boundary(mesh) || twin.on_boundary(mesh) {
                        he.data_mut(mesh).new_pos = Some(pos_v1.midpoint(pos_v2));
                    } else {
                        let pos_f1 = he.face(mesh).data(mesh).new_pos;
                        let pos_f2 = twin.face(mesh).data(mesh).new_pos;
                        if pos_f1.is_some() && pos_f2.is_some() {
                            let pos_f1 = pos_f1.unwrap();
                            let pos_f2 = pos_f2.unwrap();
                            he.data_mut(mesh).new_pos = Some(pos_v1.add_element_wise(pos_v2).add_element_wise(pos_f1).add_element_wise(pos_f2) / 4.0);
                        } else {
                            he.data_mut(mesh).new_pos = Some(pos_v1.midpoint(pos_v2));
                        }
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
                    let pos_v = v.data(mesh).pos;
                    if v.on_boundary(mesh) {
                        let mut he = v.halfedge(mesh);
                        loop {
                            he = he.twin(mesh);
                            if he.on_boundary(mesh) {
                                break;
                            }
                            he = he.next(mesh);
                        }
                        let pos_e1 = he.vertex(mesh).data(mesh).pos;
                        let pos_e2 = he.next(mesh).next(mesh).vertex(mesh).data(mesh).pos;
                        v.data_mut(mesh).new_pos = Some((0.75 * pos_v).add_element_wise(0.125 * pos_e1).add_element_wise(0.125 * pos_e2));
                    } else {
                        let mut count = 0.0;
                        let mut sum = Point3::new(0.0, 0.0, 0.0);
                        let mut vhe = he;
                        loop {
                            let twin = vhe.twin(mesh);
                            count += 1.0;
                            sum.add_assign_element_wise(twin.vertex(mesh).data(mesh).pos);
                            if let Some(pos_f) = vhe.face(mesh).data(mesh).new_pos {
                                sum.add_assign_element_wise(pos_f);
                            } else {
                                sum.add_assign_element_wise(pos_v);
                            }
                            vhe = twin.next(mesh);
                            if vhe == he {
                                break;
                            }
                        }
                        v.data_mut(mesh).new_pos = Some(((count - 2.0) * pos_v).add_element_wise(sum / count) / count);
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
                    he.data_mut(mesh).new_vert = Some(mesh.create_vertex(VData { pos: he.data(mesh).new_pos.unwrap(), new_pos: None }));
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
            *he.data_mut(mesh) = EData::default();
            let new_edge = mesh.create_edge(&he.vertex(mesh), &ev, EData::default());
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
            let fv = mesh.create_vertex(VData { pos: face.data(mesh).new_pos.unwrap(), new_pos: None });
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
                let j = if i == 0 {
                    2 * count - 1
                } else {
                    2 * i - 1
                };
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

        let face_deg = face.degree(mesh);

        let mut he = face.halfedge(mesh);
        let mut is_regular = face_deg == 4;
        while is_regular {
            if he.vertex(mesh).degree(mesh) != 4 {
                is_regular = false;
            } else {
                he = he.next(mesh);
                if he == face.halfedge(mesh) {
                    break;
                }
            }
        }

        face.data_mut(mesh).is_regular = is_regular;
        if !is_regular {
            patches.push(get_gregory_patch(face, mesh, material));
        } else {
            patches.push(get_bezier_patch(face, mesh, material));
        }
    }

    Box::new(BvhAccel::new(patches, 4, 16))
}

fn get_bezier_patch(face: &halfedge::FaceRef, mesh: &Mesh, material: &Arc<dyn Material>) -> Box<dyn Primitive> {
    let mut control_points = [[Point3::new(0.0, 0.0, 0.0); 4]; 4];

    let he = face.halfedge(mesh);
    control_points[1][1] = he.vertex(mesh).data(mesh).pos;
    let he2 = he.twin(mesh).next(mesh).twin(mesh);
    control_points[0][1] = he2.vertex(mesh).data(mesh).pos;
    let he2 = he2.next(mesh).next(mesh);
    control_points[1][0] = he2.vertex(mesh).data(mesh).pos;
    let he2 = he2.next(mesh);
    control_points[0][0] = he2.vertex(mesh).data(mesh).pos;

    let he = he.next(mesh);
    control_points[1][2] = he.vertex(mesh).data(mesh).pos;
    let he2 = he.twin(mesh).next(mesh).twin(mesh);
    control_points[1][3] = he2.vertex(mesh).data(mesh).pos;
    let he2 = he2.next(mesh).next(mesh);
    control_points[0][2] = he2.vertex(mesh).data(mesh).pos;
    let he2 = he2.next(mesh);
    control_points[0][3] = he2.vertex(mesh).data(mesh).pos;

    let he = he.next(mesh);
    control_points[2][2] = he.vertex(mesh).data(mesh).pos;
    let he2 = he.twin(mesh).next(mesh).twin(mesh);
    control_points[3][2] = he2.vertex(mesh).data(mesh).pos;
    let he2 = he2.next(mesh).next(mesh);
    control_points[2][3] = he2.vertex(mesh).data(mesh).pos;
    let he2 = he2.next(mesh);
    control_points[3][3] = he2.vertex(mesh).data(mesh).pos;

    let he = he.next(mesh);
    control_points[2][1] = he.vertex(mesh).data(mesh).pos;
    let he2 = he.twin(mesh).next(mesh).twin(mesh);
    control_points[2][0] = he2.vertex(mesh).data(mesh).pos;
    let he2 = he2.next(mesh).next(mesh);
    control_points[3][1] = he2.vertex(mesh).data(mesh).pos;
    let he2 = he2.next(mesh);
    control_points[3][0] = he2.vertex(mesh).data(mesh).pos;

    let trans_mat = [
        [1.0 / 6.0, 4.0 / 6.0, 1.0 / 6.0, 0.0],
        [0.0, 4.0 / 6.0, 2.0 / 6.0, 0.0],
        [0.0, 2.0 / 6.0, 4.0 / 6.0, 0.0],
        [0.0, 1.0 / 6.0, 4.0 / 6.0, 1.0 / 6.0],
    ];

    let mut control_points_temp = [[Point3::new(0.0, 0.0, 0.0); 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                control_points_temp[i][j].add_assign_element_wise(control_points[i][k] * trans_mat[j][k]);
            }
        }
    }

    for i in 0..4 {
        for j in 0..4 {
            control_points[j][i] = Point3::new(0.0, 0.0, 0.0);
            for k in 0..4 {
                control_points[j][i].add_assign_element_wise(control_points_temp[k][i] * trans_mat[j][k]);
            }
        }
    }

    Box::new(CubicBezier::new(control_points, material.clone()))
}

fn get_gregory_patch(face: &halfedge::FaceRef, mesh: &Mesh, material: &Arc<dyn Material>) -> Box<dyn Primitive> {
    let mut control_points = [[Point3::new(0.0, 0.0, 0.0); 4]; 4];

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

    let f0_pos = calc_face_control_points_pos(pos0, e0_pos, e1_neg, &edge_points0, &face_points0, n0, n1);
    let f0_neg = calc_face_control_points_neg(pos0, e0_neg, e3_pos, &edge_points0, &face_points0, n0, n3);
    control_points[1][1] = f0_pos.midpoint(f0_neg);

    let f1_pos = calc_face_control_points_pos(pos1, e1_pos, e2_neg, &edge_points1, &face_points1, n1, n2);
    let f1_neg = calc_face_control_points_neg(pos1, e1_neg, e0_pos, &edge_points1, &face_points1, n1, n0);
    control_points[1][2] = f1_pos.midpoint(f1_neg);

    let f2_pos = calc_face_control_points_pos(pos2, e2_pos, e3_neg, &edge_points2, &face_points2, n2, n3);
    let f2_neg = calc_face_control_points_neg(pos2, e2_neg, e1_pos, &edge_points2, &face_points2, n2, n1);
    control_points[2][2] = f2_pos.midpoint(f2_neg);

    let f3_pos = calc_face_control_points_pos(pos3, e3_pos, e0_neg, &edge_points3, &face_points3, n3, n0);
    let f3_neg = calc_face_control_points_neg(pos3, e3_neg, e2_pos, &edge_points3, &face_points3, n3, n2);
    control_points[2][1] = f3_pos.midpoint(f3_neg);

    Box::new(CubicBezier::new(control_points, material.clone()))
}

fn get_edge_points_and_face_points(vert: &halfedge::VertexRef, face: &halfedge::FaceRef, mesh: &Mesh) -> (Vec<Point3<f32>>, Vec<Point3<f32>>) {
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
        let pos_e = twin.vertex(mesh).data(mesh).pos.midpoint(pos_v);
        edge_points.push(pos_e);

        let mut pos_f = Point3::new(0.0, 0.0, 0.0);
        let mut fhe = he;
        let mut count_f = 0.0;
        loop {
            pos_f.add_assign_element_wise(fhe.vertex(mesh).data(mesh).pos);
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

fn calc_vertex_control_point(pos_v: Point3<f32>, edge_points: &[Point3<f32>], face_points: &[Point3<f32>]) -> Point3<f32> {
    let mut sum_ef = Point3::new(0.0, 0.0, 0.0);
    for pos_e in edge_points {
        sum_ef.add_assign_element_wise(*pos_e);
    }
    for pos_f in face_points {
        sum_ef.add_assign_element_wise(*pos_f);
    }

    let n = edge_points.len() as f32;
    let n_inv = 1.0 / n;
    let n5_inv = 1.0 / (n + 5.0);

    let pos = ((n - 3.0) * n5_inv * pos_v).add_element_wise(4.0 * n_inv * n5_inv * sum_ef);
    pos
}

fn calc_edge_control_points(pos_v: Point3<f32>, edge_points: &[Point3<f32>], face_points: &[Point3<f32>]) -> (Point3<f32>, Point3<f32>) {
    let n = edge_points.len() as f32;
    let n_inv = 1.0 / n;

    let frac_pi_n = std::f32::consts::PI * n_inv;
    let frac_pi_n_cos = frac_pi_n.cos();
    let frac_2pi_n = 2.0 * std::f32::consts::PI * n_inv;

    let sigma = 1.0 / (4.0 + frac_pi_n_cos * frac_pi_n_cos).sqrt();
    let temp = frac_2pi_n.cos();
    let lambda = (5.0 + temp + frac_pi_n_cos * (18.0 + 2.0 * temp).sqrt()) / 24.0;

    let mut tangent = Vector3::zero();
    let mut bitangent = Vector3::zero();
    let ka_common = 1.0 - sigma * frac_pi_n_cos;
    let kb_common = 2.0 * sigma;
    for i in 0..edge_points.len() {
        let ti = i as f32;
        let ka = ka_common * (frac_2pi_n * ti).cos();
        let kb = kb_common * (frac_2pi_n * ti + frac_pi_n).cos();
        tangent += ka * point3_as_vector3(edge_points[i]) + kb * point3_as_vector3(face_points[i]);

        let bi = ti - 1.0;
        let ka = ka_common * (frac_2pi_n * bi).cos();
        let kb = kb_common * (frac_2pi_n * bi + frac_pi_n).cos();
        bitangent += ka * point3_as_vector3(edge_points[i]) + kb * point3_as_vector3(face_points[i]);
    }
    let tangent = tangent * 2.0 * n_inv;
    let bitangent = bitangent * 2.0 * n_inv;

    let e_pos = pos_v + lambda * tangent;
    let e_neg = pos_v + lambda * bitangent;

    (e_pos, e_neg)

}

fn calc_face_control_points_pos(
    pos0: Point3<f32>,
    e0_pos: Point3<f32>,
    e1_neg: Point3<f32>,
    edge_points0: &[Point3<f32>],
    face_points0: &[Point3<f32>],
    n0: f32,
    n1: f32,
) -> Point3<f32> {
    let r = (edge_points0[edge_points0.len() - 1] - edge_points0[1]) / 3.0 + 2.0 * (face_points0[0] - face_points0[face_points0.len() - 1]) / 3.0;
    let c0 = (2.0 * std::f32::consts::PI / n0).cos();
    let c1 = (2.0 * std::f32::consts::PI / n1).cos();
    ((c1 * pos0).add_element_wise((3.0 - 2.0 * c0 - c1) * e0_pos).add_element_wise(2.0 * c0 * e1_neg) + r) / 3.0
}

fn calc_face_control_points_neg(
    pos0: Point3<f32>,
    e0_neg: Point3<f32>,
    e3_pos: Point3<f32>,
    edge_points0: &[Point3<f32>],
    face_points0: &[Point3<f32>],
    n0: f32,
    n3: f32,
) -> Point3<f32> {
    let r = (edge_points0[0] - edge_points0[2]) / 3.0 + 2.0 * (face_points0[0] - face_points0[1]) / 3.0;
    let c0 = (2.0 * std::f32::consts::PI / n0).cos();
    let c1 = (2.0 * std::f32::consts::PI / n3).cos();
    ((c1 * pos0).add_element_wise((3.0 - 2.0 * c0 - c1) * e0_neg).add_element_wise(2.0 * c0 * e3_pos) + r) / 3.0
}

fn point3_as_vector3(p: Point3<f32>) -> Vector3<f32> {
    Vector3::new(p.x, p.y, p.z)
}