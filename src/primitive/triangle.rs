use crate::core::bbox::Bbox;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::medium::Medium;
use crate::core::primitive::Primitive;
use crate::core::ray::Ray;
use cgmath::{InnerSpace, Zero};
use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct MeshVertex {
    pub position: cgmath::Point3<f32>,
    pub normal: cgmath::Vector3<f32>,
    pub texcoords: cgmath::Point2<f32>,
    pub tangent: cgmath::Vector3<f32>,
    pub bitangent: cgmath::Vector3<f32>,
}

pub struct TriangleMesh {
    vertices: Vec<MeshVertex>,
    indices: Vec<u32>,
    material: Arc<dyn Material>,
    inside_medium: Option<Arc<dyn Medium>>,
}

pub struct Triangle {
    mesh: Arc<TriangleMesh>,
    indices: [usize; 3],
    bbox: Bbox,
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            position: cgmath::Point3::new(0.0, 0.0, 0.0),
            normal: cgmath::Vector3::unit_z(),
            texcoords: cgmath::Point2::new(0.0, 0.0),
            tangent: cgmath::Vector3::unit_x(),
            bitangent: cgmath::Vector3::unit_y(),
        }
    }
}

impl TriangleMesh {
    pub fn new(
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
        material: Arc<dyn Material>,
        inside_medium: Option<Arc<dyn Medium>>,
    ) -> Self {
        let mut mesh = Self {
            vertices,
            indices,
            material,
            inside_medium,
        };
        mesh.calc_tangents();
        mesh
    }

    pub fn position(&self, index: usize) -> cgmath::Point3<f32> {
        self.vertices[index].position
    }

    pub fn normal(&self, index: usize) -> cgmath::Vector3<f32> {
        self.vertices[index].normal
    }

    pub fn texcoords(&self, index: usize) -> cgmath::Point2<f32> {
        self.vertices[index].texcoords
    }

    pub fn tangent(&self, index: usize) -> cgmath::Vector3<f32> {
        self.vertices[index].tangent
    }

    pub fn bitangent(&self, index: usize) -> cgmath::Vector3<f32> {
        self.vertices[index].bitangent
    }

    pub fn into_triangles(self) -> Vec<Box<dyn Primitive>> {
        let rc = Arc::new(self);
        let triangle_count = rc.indices.len() / 3;
        let mut triangles = vec![];
        for i in 0..triangle_count {
            let i0 = rc.indices[3 * i] as usize;
            let i1 = rc.indices[3 * i + 1] as usize;
            let i2 = rc.indices[3 * i + 2] as usize;
            triangles.push(Box::new(Triangle::new(rc.clone(), [i0, i1, i2])) as Box<dyn Primitive>);
        }
        triangles
    }
}

impl Triangle {
    fn new(mesh: Arc<TriangleMesh>, indices: [usize; 3]) -> Self {
        let p0 = mesh.position(indices[0]);
        let p1 = mesh.position(indices[1]);
        let p2 = mesh.position(indices[2]);
        let bbox = Bbox::from_points(&[p0, p1, p2]);
        Self {
            mesh,
            indices,
            bbox,
        }
    }

    fn intersect_ray(&self, ray: &Ray) -> Option<(f32, f32, f32, f32)> {
        let p0 = self.mesh.position(self.indices[0]);
        let p1 = self.mesh.position(self.indices[1]);
        let p2 = self.mesh.position(self.indices[2]);
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

impl Primitive for Triangle {
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
                    let n0 = self.mesh.normal(self.indices[0]);
                    let n1 = self.mesh.normal(self.indices[1]);
                    let n2 = self.mesh.normal(self.indices[2]);
                    (n0 * u + n1 * v + n2 * w).normalize()
                };
                inter.texcoords = {
                    let uv0 = self.mesh.texcoords(self.indices[0]);
                    let uv1 = self.mesh.texcoords(self.indices[1]);
                    let uv2 = self.mesh.texcoords(self.indices[2]);
                    lerp_point2(uv0, uv1, uv2, u, v, w)
                };
                inter.tangent = {
                    let t0 = self.mesh.tangent(self.indices[0]);
                    let t1 = self.mesh.tangent(self.indices[1]);
                    let t2 = self.mesh.tangent(self.indices[2]);
                    t0 * u + t1 * v + t2 * w
                };
                inter.bitangent = {
                    let b0 = self.mesh.bitangent(self.indices[0]);
                    let b1 = self.mesh.bitangent(self.indices[1]);
                    let b2 = self.mesh.bitangent(self.indices[2]);
                    b0 * u + b1 * v + b2 * w
                };
                inter.primitive = Some(self);
                return true;
            }
        }
        false
    }

    fn bbox(&self) -> Bbox {
        self.bbox
    }

    fn material(&self) -> Option<&dyn Material> {
        Some(&*self.mesh.material)
    }

    fn inside_medium(&self) -> Option<&dyn Medium> {
        self.mesh.inside_medium.as_ref().map(|rc| rc.as_ref())
    }
}

impl TriangleMesh {
    fn calc_tangents(&mut self) {
        let vertex_count = self.vertices.len();
        let mut tangents_sum = vec![cgmath::Vector3::zero(); vertex_count];
        let mut tangents_cnt = vec![0; vertex_count];
        let mut bitangents_sum = vec![cgmath::Vector3::zero(); vertex_count];
        let mut bitangents_cnt = vec![0; vertex_count];

        let triangle_count = self.indices.len() / 3;
        for i in 0..triangle_count {
            let i0 = self.indices[3 * i] as usize;
            let i1 = self.indices[3 * i + 1] as usize;
            let i2 = self.indices[3 * i + 2] as usize;

            let p0 = self.vertices[i0].position;
            let p1 = self.vertices[i1].position;
            let p2 = self.vertices[i2].position;
            let e1 = p1 - p0;
            let e2 = p2 - p0;

            let uv0 = self.vertices[i0].texcoords;
            let uv1 = self.vertices[i1].texcoords;
            let uv2 = self.vertices[i2].texcoords;
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
                self.vertices[i].tangent = tangents_sum[i] / tangents_cnt[i] as f32;
            }
            if bitangents_cnt[i] != 0 {
                self.vertices[i].bitangent = bitangents_sum[i] /  bitangents_cnt[i] as f32;
            }
        }
    }
}

fn lerp_point2(
    p0: cgmath::Point2<f32>,
    p1: cgmath::Point2<f32>,
    p2: cgmath::Point2<f32>,
    u: f32,
    v: f32,
    w: f32,
) -> cgmath::Point2<f32> {
    let x = p0.x * u + p1.x * v + p2.x * w;
    let y = p0.y * u + p1.y * v + p2.y * w;
    cgmath::Point2::new(x, y)
}
