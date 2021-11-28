use std::sync::Arc;

use crate::core::{
    bbox::Bbox, intersection::Intersection, loader::InputParams, ray::Ray, rng::Rng, scene::Scene,
};

use super::{BasicPrimitiveRef, BvhAccel, PrimitiveT};

#[derive(Copy, Clone)]
pub struct MeshVertex {
    pub position: glam::Vec3A,
    pub normal: glam::Vec3A,
    pub texcoords: glam::Vec2,
    pub tangent: glam::Vec3A,
    pub bitangent: glam::Vec3A,
}

pub struct TriMesh {
    triangles: BvhAccel<Triangle>,
}

pub struct Triangle {
    vertices: Arc<Vec<MeshVertex>>,
    indices: [usize; 3],
    bbox: Bbox,
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            position: glam::Vec3A::ZERO,
            normal: glam::Vec3A::Z,
            texcoords: glam::Vec2::ZERO,
            tangent: glam::Vec3A::X,
            bitangent: glam::Vec3A::Y,
        }
    }
}

impl TriMesh {
    pub fn new(mut vertices: Vec<MeshVertex>, indices: Vec<u32>) -> Self {
        let vertex_count = vertices.len();
        let mut tangents_sum = vec![glam::Vec3A::ZERO; vertex_count];
        let mut tangents_cnt = vec![0; vertex_count];
        let mut bitangents_sum = vec![glam::Vec3A::ZERO; vertex_count];
        let mut bitangents_cnt = vec![0; vertex_count];

        let triangle_count = indices.len() / 3;
        for i in 0..triangle_count {
            let i0 = indices[3 * i] as usize;
            let i1 = indices[3 * i + 1] as usize;
            let i2 = indices[3 * i + 2] as usize;

            let p0 = vertices[i0].position;
            let p1 = vertices[i1].position;
            let p2 = vertices[i2].position;
            let e1 = p1 - p0;
            let e2 = p2 - p0;

            let uv0 = vertices[i0].texcoords;
            let uv1 = vertices[i1].texcoords;
            let uv2 = vertices[i2].texcoords;
            let u1 = uv1 - uv0;
            let u2 = uv2 - uv0;

            let det = u1.x * u2.y - u1.y * u2.x;
            if det != 0.0 {
                let det = 1.0 / det;
                let t = ((e1 * u2.y - e2 * u1.y) * det).normalize();
                tangents_sum[i0] += t;
                tangents_cnt[i0] += 1;
                tangents_sum[i1] += t;
                tangents_cnt[i1] += 1;
                tangents_sum[i2] += t;
                tangents_cnt[i2] += 1;
                let b = ((e2 * u1.x - e1 * u2.x) * det).normalize();
                bitangents_sum[i0] += b;
                bitangents_cnt[i0] += 1;
                bitangents_sum[i1] += b;
                bitangents_cnt[i1] += 1;
                bitangents_sum[i2] += b;
                bitangents_cnt[i2] += 1;
            }
        }

        for i in 0..vertex_count {
            if tangents_cnt[i] != 0 {
                vertices[i].tangent = (tangents_sum[i] / tangents_cnt[i] as f32).normalize();
            }
            if bitangents_cnt[i] != 0 {
                vertices[i].bitangent = (bitangents_sum[i] / bitangents_cnt[i] as f32).normalize();
            }
        }

        let vertices = Arc::new(vertices);
        let triangle_count = indices.len() / 3;
        let mut triangles = Vec::with_capacity(triangle_count);
        for i in 0..triangle_count {
            let i0 = indices[3 * i] as usize;
            let i1 = indices[3 * i + 1] as usize;
            let i2 = indices[3 * i + 2] as usize;
            triangles.push(Arc::new(Triangle::new(vertices.clone(), [i0, i1, i2])));
        }
        let triangles = BvhAccel::new(triangles, 4, 16);

        Self { triangles }
    }

    pub fn load(_scene: &Scene, params: &mut InputParams) -> anyhow::Result<Self> {
        let obj_file = params.get_file_path("obj_file")?;

        let mut load_options = tobj::LoadOptions::default();
        load_options.triangulate = true;
        load_options.single_index = true;
        let (models, _) = tobj::load_obj(obj_file, &load_options)?;

        let mut vertices = vec![];
        let mut indices = vec![];
        for model in models {
            let vertex_count = model.mesh.positions.len() / 3;
            let mut model_vertices = vec![MeshVertex::default(); vertex_count];
            for i in 0..vertex_count {
                let i0 = 3 * i;
                let i1 = 3 * i + 1;
                let i2 = 3 * i + 2;
                if i2 < model.mesh.positions.len() {
                    model_vertices[i].position = glam::Vec3A::new(
                        model.mesh.positions[i0],
                        model.mesh.positions[i1],
                        model.mesh.positions[i2],
                    );
                }
                if i2 < model.mesh.normals.len() {
                    model_vertices[i].normal = glam::Vec3A::new(
                        model.mesh.normals[i0],
                        model.mesh.normals[i1],
                        model.mesh.normals[i2],
                    );
                }
                if 2 * i + 1 < model.mesh.texcoords.len() {
                    model_vertices[i].texcoords = glam::Vec2::new(
                        model.mesh.texcoords[2 * i],
                        model.mesh.texcoords[2 * i + 1],
                    );
                }
            }
            vertices.append(&mut model_vertices);
            let mut model_indices = model
                .mesh
                .indices
                .into_iter()
                .map(|ind| ind + indices.len() as u32)
                .collect::<Vec<_>>();
            indices.append(&mut model_indices);
        }

        Ok(TriMesh::new(vertices, indices))
    }
}

impl Triangle {
    fn new(vertices: Arc<Vec<MeshVertex>>, indices: [usize; 3]) -> Self {
        let p0 = vertices[indices[0]].position;
        let p1 = vertices[indices[1]].position;
        let p2 = vertices[indices[2]].position;
        let bbox = Bbox::from_points(&[p0, p1, p2]);
        Self {
            vertices,
            indices,
            bbox,
        }
    }

    fn intersect_ray(&self, ray: &Ray) -> Option<(f32, f32, f32, f32)> {
        let p0 = self.vertices[self.indices[0]].position;
        let p1 = self.vertices[self.indices[1]].position;
        let p2 = self.vertices[self.indices[2]].position;
        let e1 = p1 - p0;
        let e2 = p2 - p0;
        let q = ray.direction.cross(e2);
        let det = e1.dot(q);
        if det != 0.0 {
            let det = 1.0 / det;
            let s = ray.origin - p0;
            let v = s.dot(q) * det;
            if v >= 0.0 {
                let r = s.cross(e1);
                let w = ray.direction.dot(r) * det;
                let u = 1.0 - v - w;
                if w >= 0.0 && u >= 0.0 {
                    let t = e2.dot(r) * det;
                    return Some((t, u, v, w));
                }
            }
        }
        None
    }
}

impl PrimitiveT for TriMesh {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        self.triangles.intersect_test(ray, t_max)
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        self.triangles.intersect(ray, inter)
    }

    fn bbox(&self) -> Bbox {
        self.triangles.bbox()
    }

    fn sample<'a>(&'a self, sampler: &mut Rng) -> (Intersection<'a>, f32) {
        self.triangles.sample(sampler)
    }

    fn pdf(&self, inter: &Intersection<'_>) -> f32 {
        self.triangles.pdf(inter)
    }
}

impl PrimitiveT for Triangle {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        if let Some((t, _, _, _)) = self.intersect_ray(ray) {
            t > ray.t_min && t < t_max
        } else {
            false
        }
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        if let Some((t, u, v, w)) = self.intersect_ray(ray) {
            if t > ray.t_min && t < inter.t {
                inter.t = t;
                inter.normal = {
                    let n0 = self.vertices[self.indices[0]].normal;
                    let n1 = self.vertices[self.indices[1]].normal;
                    let n2 = self.vertices[self.indices[2]].normal;
                    (n0 * u + n1 * v + n2 * w).normalize()
                };
                inter.texcoords = {
                    let uv0 = self.vertices[self.indices[0]].texcoords;
                    let uv1 = self.vertices[self.indices[1]].texcoords;
                    let uv2 = self.vertices[self.indices[2]].texcoords;
                    lerp_point2(uv0, uv1, uv2, u, v, w)
                };
                inter.tangent = {
                    let t0 = self.vertices[self.indices[0]].tangent;
                    let t1 = self.vertices[self.indices[1]].tangent;
                    let t2 = self.vertices[self.indices[2]].tangent;
                    t0 * u + t1 * v + t2 * w
                };
                inter.bitangent = {
                    let b0 = self.vertices[self.indices[0]].bitangent;
                    let b1 = self.vertices[self.indices[1]].bitangent;
                    let b2 = self.vertices[self.indices[2]].bitangent;
                    b0 * u + b1 * v + b2 * w
                };
                inter.primitive = Some(BasicPrimitiveRef::Triangle(self));
                return true;
            }
        }
        false
    }

    fn bbox(&self) -> Bbox {
        self.bbox
    }

    fn sample<'a>(&'a self, sampler: &mut Rng) -> (Intersection<'a>, f32) {
        let rand = sampler.uniform_2d();
        let r0_sqrt = rand.0.sqrt();
        let u = 1.0 - r0_sqrt;
        let v = r0_sqrt * (1.0 - rand.1);
        let w = 1.0 - u - v;

        let p0 = self.vertices[self.indices[0]].position;
        let p1 = self.vertices[self.indices[1]].position;
        let p2 = self.vertices[self.indices[2]].position;

        let n0 = self.vertices[self.indices[0]].normal;
        let n1 = self.vertices[self.indices[1]].normal;
        let n2 = self.vertices[self.indices[2]].normal;

        let t0 = self.vertices[self.indices[0]].tangent;
        let t1 = self.vertices[self.indices[1]].tangent;
        let t2 = self.vertices[self.indices[2]].tangent;

        let b0 = self.vertices[self.indices[0]].bitangent;
        let b1 = self.vertices[self.indices[1]].bitangent;
        let b2 = self.vertices[self.indices[2]].bitangent;

        let uv0 = self.vertices[self.indices[0]].texcoords;
        let uv1 = self.vertices[self.indices[1]].texcoords;
        let uv2 = self.vertices[self.indices[2]].texcoords;

        let p = p0 * u + p1 * v + p2 * w;
        let area = (p1 - p0).cross(p2 - p0).length() * 0.5;

        let norm = n0 * u + n1 * v + n2 * w;
        let tan = t0 * u + t1 * v + t2 * w;
        let bitan = b0 * u + b1 * v + b2 * w;
        let uv = uv0 * u + uv1 * v + uv2 * w;

        let inter = Intersection {
            position: p,
            normal: norm,
            tangent: tan,
            bitangent: bitan,
            texcoords: uv,
            primitive: Some(BasicPrimitiveRef::Triangle(self)),
            ..Default::default()
        };

        (inter, 1.0 / area.max(0.001))
    }

    fn pdf(&self, _inter: &Intersection<'_>) -> f32 {
        let p0 = self.vertices[self.indices[0]].position;
        let p1 = self.vertices[self.indices[1]].position;
        let p2 = self.vertices[self.indices[2]].position;

        let area = (p1 - p0).cross(p2 - p0).length() * 0.5;

        1.0 / area.max(0.001)
    }
}

fn lerp_point2(
    p0: glam::Vec2,
    p1: glam::Vec2,
    p2: glam::Vec2,
    u: f32,
    v: f32,
    w: f32,
) -> glam::Vec2 {
    let x = p0.x * u + p1.x * v + p2.x * w;
    let y = p0.y * u + p1.y * v + p2.y * w;
    glam::Vec2::new(x, y)
}
