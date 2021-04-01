use crate::core::bbox::Bbox;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::primitive::Primitive;
use crate::core::ray::Ray;
use cgmath::InnerSpace;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct MeshVertex {
    pub position: cgmath::Point3<f32>,
    pub normal: cgmath::Vector3<f32>,
    pub texcoords: cgmath::Point2<f32>,
}

pub struct TriangleMesh {
    vertices: Vec<MeshVertex>,
    indices: Vec<u32>,
    material: Rc<dyn Material>,
}

pub struct Triangle {
    mesh: Rc<TriangleMesh>,
    indices: [usize; 3],
    bbox: Bbox,
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            position: cgmath::Point3::new(0.0, 0.0, 0.0),
            normal: cgmath::Vector3::unit_y(),
            texcoords: cgmath::Point2::new(0.0, 0.0),
        }
    }
}

impl TriangleMesh {
    pub fn new(vertices: Vec<MeshVertex>, indices: Vec<u32>, material: Rc<dyn Material>) -> Self {
        Self {
            vertices,
            indices,
            material,
        }
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

    pub fn into_triangles(self) -> Vec<Box<dyn Primitive>> {
        let rc = Rc::new(self);
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
    fn new(mesh: Rc<TriangleMesh>, indices: [usize; 3]) -> Self {
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
            let u = s.dot(q) * det;
            if u >= 0.0 {
                let r = s.cross(e1);
                let v = ray.direction.dot(r) * det;
                let w = 1.0 - u - v;
                if v >= 0.0 && w >= 0.0 {
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
                if inter.normal.dot(ray.direction) > 0.0 {
                    inter.normal = -inter.normal;
                }
                inter.texcoords = {
                    let uv0 = self.mesh.texcoords(self.indices[0]);
                    let uv1 = self.mesh.texcoords(self.indices[1]);
                    let uv2 = self.mesh.texcoords(self.indices[2]);
                    lerp_point2(uv0, uv1, uv2, u, v, w)
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
