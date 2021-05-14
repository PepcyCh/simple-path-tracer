use std::ptr::NonNull;

use cgmath::{InnerSpace, Matrix, Matrix2, SquareMatrix, Vector2};

#[derive(Clone, Copy)]
pub struct PndfGaussTerm {
    pub u: Vector2<f32>,
    pub s: Vector2<f32>,
    pub jacobian: Matrix2<f32>,
    pub jacobian_t: Matrix2<f32>,
}

struct Tuple4fBbox {
    min: (f32, f32, f32, f32),
    max: (f32, f32, f32, f32),
}

struct PndfBvhNode {
    lc: Option<Box<PndfBvhNode>>,
    rc: Option<Box<PndfBvhNode>>,
    bbox: Tuple4fBbox,
    start: usize,
    end: usize,
}

struct PndfBvh {
    bvh_root: Option<Box<PndfBvhNode>>,
    terms: Vec<NonNull<PndfGaussTerm>>,
}

struct Tuple2fBbox {
    min: (f32, f32),
    max: (f32, f32),
}

struct PndfUvBvhNode {
    lc: Option<Box<PndfUvBvhNode>>,
    rc: Option<Box<PndfUvBvhNode>>,
    bbox: Tuple2fBbox,
    start: usize,
    end: usize,
}

struct PndfUvBvh {
    bvh_root: Option<Box<PndfUvBvhNode>>,
    terms: Vec<NonNull<PndfGaussTerm>>,
}

pub struct PndfAccel {
    s_block_count: usize,
    bvhs: Vec<PndfBvh>,
    uv_bvh: PndfUvBvh,
}

impl PndfAccel {
    pub fn new(
        mut terms: Vec<&mut PndfGaussTerm>,
        max_leaf_size: usize,
        s_block_count: usize,
    ) -> Self {
        let bvh_count = s_block_count * s_block_count;
        let mut terms_split: Vec<_> = (0..bvh_count).into_iter().map(|_| vec![]).collect();

        for term in &mut terms {
            let x = ((term.s.x * s_block_count as f32) as usize).min(s_block_count - 1);
            let y = ((term.s.y * s_block_count as f32) as usize).min(s_block_count - 1);
            let term: *mut PndfGaussTerm = *term;
            terms_split[x * s_block_count + y].push(NonNull::new(term).unwrap());
        }

        let bvhs: Vec<_> = terms_split
            .into_iter()
            .map(|terms| PndfBvh::new(terms, max_leaf_size))
            .collect();

        let terms_ptr: Vec<_> = terms
            .into_iter()
            .map(|term| {
                let term: *mut PndfGaussTerm = term;
                NonNull::new(term).unwrap()
            })
            .collect();
        let uv_bvh = PndfUvBvh::new(terms_ptr, max_leaf_size);

        Self {
            s_block_count,
            bvhs,
            uv_bvh,
        }
    }

    pub fn calc(
        &self,
        sigma_p: f32,
        sigma_hx: f32,
        sigma_hy: f32,
        sigma_r: f32,
        u: Vector2<f32>,
        s: Vector2<f32>,
    ) -> f32 {
        let x = ((s.x * self.s_block_count as f32) as usize).min(self.s_block_count - 1);
        let y = ((s.y * self.s_block_count as f32) as usize).min(self.s_block_count - 1);
        let bvh_ind = x * self.s_block_count + y;
        self.bvhs[bvh_ind].calc(sigma_p, sigma_hx, sigma_hy, sigma_r, u, s)
    }

    pub fn find_term(&self, u: Vector2<f32>) -> PndfGaussTerm {
        self.uv_bvh.find_term(u)
    }
}

impl PndfBvh {
    fn new(terms: Vec<NonNull<PndfGaussTerm>>, max_leaf_size: usize) -> Self {
        if terms.is_empty() {
            return Self {
                bvh_root: None,
                terms,
            };
        }

        let term0 = unsafe { terms[0].as_ref() };
        let mut us_min = term0.us_tuple();
        let mut us_max = term0.us_tuple();
        terms.iter().skip(1).for_each(|term| {
            let term = unsafe { term.as_ref() };
            let tuple = term.us_tuple();
            us_min = min_tuple4f(us_min, tuple);
            us_max = max_tuple4f(us_max, tuple);
        });
        let mut bvh_root = Box::new(PndfBvhNode::new(
            0,
            terms.len(),
            Tuple4fBbox::new(us_min, us_max),
        ));

        let mut stack = vec![&mut bvh_root];
        while let Some(u) = stack.pop() {
            if u.size() < max_leaf_size {
                continue;
            }

            let mid = u.start + u.size() / 2;

            let term_start = unsafe { terms[u.start].as_ref() };
            let mut us_min = term_start.us_tuple();
            let mut us_max = term_start.us_tuple();
            for i in u.start..mid {
                let term = unsafe { terms[i].as_ref() };
                let tuple = term.us_tuple();
                us_min = min_tuple4f(us_min, tuple);
                us_max = max_tuple4f(us_max, tuple);
            }
            let lc = Box::new(PndfBvhNode::new(
                u.start,
                mid,
                Tuple4fBbox::new(us_min, us_max),
            ));

            let term_mid = unsafe { terms[mid].as_ref() };
            let mut us_min = term_mid.us_tuple();
            let mut us_max = term_mid.us_tuple();
            for i in mid..u.end {
                let term = unsafe { terms[i].as_ref() };
                let tuple = term.us_tuple();
                us_min = min_tuple4f(us_min, tuple);
                us_max = max_tuple4f(us_max, tuple);
            }
            let rc = Box::new(PndfBvhNode::new(
                mid,
                u.end,
                Tuple4fBbox::new(us_min, us_max),
            ));

            u.lc = Some(lc);
            u.rc = Some(rc);
            stack.push(u.lc.as_mut().unwrap());
            stack.push(u.rc.as_mut().unwrap());
        }

        Self {
            bvh_root: Some(bvh_root),
            terms,
        }
    }

    fn calc(
        &self,
        sigma_p: f32,
        sigma_hx: f32,
        sigma_hy: f32,
        sigma_r: f32,
        u: Vector2<f32>,
        s: Vector2<f32>,
    ) -> f32 {
        if self.bvh_root.is_none() {
            return 0.0;
        }

        let us_tuple = (u.x, u.y, s.x, s.y);

        let mut value = 0.0;
        let mut stack = vec![self.bvh_root.as_ref().unwrap()];
        while let Some(curr) = stack.pop() {
            let dist = curr.bbox.dist_to_point(us_tuple);
            if dist.0 > 3.0 * sigma_hx
                || dist.1 > 3.0 * sigma_hy
                || dist.2 > 3.0 * sigma_r
                || dist.3 > 3.0 * sigma_r
            {
                continue;
            }

            if curr.is_leaf() {
                for i in curr.start..curr.end {
                    let term = unsafe { self.terms[i].as_ref() };
                    value += term.calc(sigma_p, sigma_hx, sigma_hy, sigma_r, u, s);
                }
            } else {
                stack.push(curr.lc.as_ref().unwrap());
                stack.push(curr.rc.as_ref().unwrap());
            }
        }

        value
    }
}

impl PndfBvhNode {
    fn new(start: usize, end: usize, bbox: Tuple4fBbox) -> Self {
        Self {
            lc: None,
            rc: None,
            bbox,
            start,
            end,
        }
    }

    fn size(&self) -> usize {
        self.end - self.start
    }

    fn is_leaf(&self) -> bool {
        self.lc.is_none()
    }
}

impl PndfUvBvh {
    fn new(terms: Vec<NonNull<PndfGaussTerm>>, max_leaf_size: usize) -> Self {
        if terms.is_empty() {
            return Self {
                bvh_root: None,
                terms,
            };
        }

        let term0 = unsafe { terms[0].as_ref() };
        let mut us_min = term0.u_tuple();
        let mut us_max = term0.u_tuple();
        terms.iter().skip(1).for_each(|term| {
            let term = unsafe { term.as_ref() };
            let tuple = term.u_tuple();
            us_min = min_tuple2f(us_min, tuple);
            us_max = max_tuple2f(us_max, tuple);
        });
        let mut bvh_root = Box::new(PndfUvBvhNode::new(
            0,
            terms.len(),
            Tuple2fBbox::new(us_min, us_max),
        ));

        let mut stack = vec![&mut bvh_root];
        while let Some(u) = stack.pop() {
            if u.size() < max_leaf_size {
                continue;
            }

            let mid = u.start + u.size() / 2;

            let term_start = unsafe { terms[u.start].as_ref() };
            let mut us_min = term_start.u_tuple();
            let mut us_max = term_start.u_tuple();
            for i in u.start..mid {
                let term = unsafe { terms[i].as_ref() };
                let tuple = term.u_tuple();
                us_min = min_tuple2f(us_min, tuple);
                us_max = max_tuple2f(us_max, tuple);
            }
            let lc = Box::new(PndfUvBvhNode::new(
                u.start,
                mid,
                Tuple2fBbox::new(us_min, us_max),
            ));

            let term_mid = unsafe { terms[mid].as_ref() };
            let mut us_min = term_mid.u_tuple();
            let mut us_max = term_mid.u_tuple();
            for i in mid..u.end {
                let term = unsafe { terms[i].as_ref() };
                let tuple = term.u_tuple();
                us_min = min_tuple2f(us_min, tuple);
                us_max = max_tuple2f(us_max, tuple);
            }
            let rc = Box::new(PndfUvBvhNode::new(
                mid,
                u.end,
                Tuple2fBbox::new(us_min, us_max),
            ));

            u.lc = Some(lc);
            u.rc = Some(rc);
            stack.push(u.lc.as_mut().unwrap());
            stack.push(u.rc.as_mut().unwrap());
        }

        Self {
            bvh_root: Some(bvh_root),
            terms,
        }
    }

    fn find_term(&self, u: Vector2<f32>) -> PndfGaussTerm {
        let u_tuple = (u.x, u.y);

        let mut min_dist_sqr = 10.0;
        let mut result = *unsafe { self.terms[0].as_ref() };
        let mut stack = vec![self.bvh_root.as_ref().unwrap()];
        while let Some(curr) = stack.pop() {
            let dist = curr.bbox.dist_to_point(u_tuple);
            let dist_sqr = dist.0 * dist.0 + dist.1 * dist.1;
            if dist_sqr > min_dist_sqr {
                continue;
            }

            if curr.is_leaf() {
                for i in curr.start..curr.end {
                    let term = unsafe { self.terms[i].as_ref() };
                    let dist_sqr = (u.x - term.u.x).exp2() + (u.y - term.u.y).exp2();
                    if dist_sqr < min_dist_sqr {
                        min_dist_sqr = dist_sqr;
                        result = *term;
                    }
                }
            } else {
                stack.push(curr.lc.as_ref().unwrap());
                stack.push(curr.rc.as_ref().unwrap());
            }
        }
        result
    }
}

unsafe impl Send for PndfBvh {}
unsafe impl Sync for PndfBvh {}

impl PndfUvBvhNode {
    fn new(start: usize, end: usize, bbox: Tuple2fBbox) -> Self {
        Self {
            lc: None,
            rc: None,
            bbox,
            start,
            end,
        }
    }

    fn size(&self) -> usize {
        self.end - self.start
    }

    fn is_leaf(&self) -> bool {
        self.lc.is_none()
    }
}

unsafe impl Send for PndfUvBvh {}
unsafe impl Sync for PndfUvBvh {}

impl PndfGaussTerm {
    pub fn new(u: Vector2<f32>, s: Vector2<f32>, jacobian: Matrix2<f32>) -> Self {
        Self {
            u,
            s,
            jacobian,
            jacobian_t: jacobian.transpose(),
        }
    }

    fn us_tuple(&self) -> (f32, f32, f32, f32) {
        (self.u.x, self.u.y, self.s.x, self.s.y)
    }

    fn u_tuple(&self) -> (f32, f32) {
        (self.u.x, self.u.y)
    }

    fn calc(
        &self,
        sigma_p: f32,
        sigma_hx: f32,
        sigma_hy: f32,
        sigma_r: f32,
        u: Vector2<f32>,
        s: Vector2<f32>,
    ) -> f32 {
        let sigma_p_sqr = sigma_p * sigma_p;
        let sigma_p_sqr_inv = 1.0 / sigma_p_sqr;
        let sigma_h_sqr = sigma_hx * sigma_hy;
        let sigma_h_sqr_inv = 1.0 / sigma_h_sqr;
        let sigma_r_sqr = sigma_r * sigma_r;
        let sigma_r_sqr_inv = 1.0 / sigma_r_sqr;

        let delta_s = s - self.s;

        let mat_a = sigma_h_sqr_inv * Matrix2::identity()
            + sigma_r_sqr_inv * self.jacobian_t * self.jacobian;
        let mat_a_inv = mat_a.invert().unwrap();
        let mat_b = -sigma_r_sqr_inv * self.jacobian;
        let mat_c = (sigma_h_sqr_inv + sigma_r_sqr_inv) * Matrix2::identity();
        let mu = -mat_a_inv * mat_b * delta_s;

        let k = (-0.5 * (delta_s.dot(mat_c * delta_s) - mu.dot(mat_a * mu))).exp();
        k * integrate_gaussian_multiplication_2d(
            u,
            sigma_p_sqr_inv * Matrix2::identity(),
            self.u + mu,
            mat_a,
        )
    }
}

impl Tuple4fBbox {
    fn new(min: (f32, f32, f32, f32), max: (f32, f32, f32, f32)) -> Self {
        Self { min, max }
    }

    fn dist_to_point(&self, p: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
        (
            (p.0 - self.max.0).max(self.min.0 - p.0).max(0.0),
            (p.1 - self.max.1).max(self.min.1 - p.1).max(0.0),
            (p.2 - self.max.2).max(self.min.2 - p.2).max(0.0),
            (p.3 - self.max.3).max(self.min.3 - p.3).max(0.0),
        )
    }
}

impl Tuple2fBbox {
    fn new(min: (f32, f32), max: (f32, f32)) -> Self {
        Self { min, max }
    }

    fn dist_to_point(&self, p: (f32, f32)) -> (f32, f32) {
        (
            (p.0 - self.max.0).max(self.min.0 - p.0).max(0.0),
            (p.1 - self.max.1).max(self.min.1 - p.1).max(0.0),
        )
    }
}

fn min_tuple4f(
    (a0, a1, a2, a3): (f32, f32, f32, f32),
    (b0, b1, b2, b3): (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
    (a0.min(b0), a1.min(b1), a2.min(b2), a3.min(b3))
}

fn max_tuple4f(
    (a0, a1, a2, a3): (f32, f32, f32, f32),
    (b0, b1, b2, b3): (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
    (a0.max(b0), a1.max(b1), a2.max(b2), a3.max(b3))
}

fn min_tuple2f((a0, a1): (f32, f32), (b0, b1): (f32, f32)) -> (f32, f32) {
    (a0.min(b0), a1.min(b1))
}

fn max_tuple2f((a0, a1): (f32, f32), (b0, b1): (f32, f32)) -> (f32, f32) {
    (a0.max(b0), a1.max(b1))
}

fn integrate_gaussian_multiplication_2d(
    mu0: Vector2<f32>,
    sigma_sqr_inv0: Matrix2<f32>,
    mu1: Vector2<f32>,
    sigma_sqr_inv1: Matrix2<f32>,
) -> f32 {
    let sigma_sqr_inv = sigma_sqr_inv0 + sigma_sqr_inv1;
    let sigma_sqr = sigma_sqr_inv.invert().unwrap();
    let sigma_sqr_sum_inv = sigma_sqr_inv0 * sigma_sqr * sigma_sqr_inv1;
    let mu_diff = mu0 - mu1;
    sigma_sqr_sum_inv.determinant() * (-0.5 * mu_diff.dot(sigma_sqr_sum_inv * mu_diff)).exp() * 0.5
        / std::f32::consts::PI
}
