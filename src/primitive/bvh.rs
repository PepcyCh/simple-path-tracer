use std::{collections::HashSet, sync::Arc};

use crate::core::{bbox::Bbox, intersection::Intersection, ray::Ray, rng::Rng};

use super::PrimitiveT;

pub struct BvhAccel<P: PrimitiveT> {
    bvh_root: Option<Box<BvhNode>>,
    primitives: Vec<Arc<P>>,
}

struct BvhNode {
    lc: Option<Box<BvhNode>>,
    rc: Option<Box<BvhNode>>,
    bbox: Bbox,
    start: usize,
    end: usize,
}

impl<P: PrimitiveT> BvhAccel<P> {
    pub fn new(mut primitives: Vec<Arc<P>>, max_leaf_size: usize, bucket_number: usize) -> Self {
        if primitives.is_empty() {
            return Self {
                bvh_root: None,
                primitives,
            };
        };

        let mut bbox = primitives[0].bbox();
        primitives.iter().skip(1).for_each(|prim| {
            bbox = bbox.merge(prim.bbox());
        });
        let mut bvh_root = Box::new(BvhNode::new(0, primitives.len(), bbox));

        let mut stack = vec![&mut bvh_root];
        while let Some(u) = stack.pop() {
            if u.size() <= max_leaf_size {
                continue;
            }

            let bbox = u.bbox;
            let len_per_bucket = (bbox.p_max - bbox.p_min) / bucket_number as f32;

            let mut boxes_x = vec![Bbox::empty(); bucket_number];
            let mut boxes_y = vec![Bbox::empty(); bucket_number];
            let mut boxes_z = vec![Bbox::empty(); bucket_number];

            let mut prim_indices_x = vec![vec![]; bucket_number];
            let mut prim_indices_y = vec![vec![]; bucket_number];
            let mut prim_indices_z = vec![vec![]; bucket_number];

            for i in u.start..u.end {
                let bbox = primitives[i].bbox();
                let centroid = bbox.centroid();

                let x = (centroid.x - bbox.p_min.x) / len_per_bucket.x;
                if x >= 0.0 && x < bucket_number as f32 {
                    let x = x as usize;
                    boxes_x[x] = boxes_x[x].merge(bbox);
                    prim_indices_x[x].push(i);
                }

                let y = (centroid.y - bbox.p_min.y) / len_per_bucket.y;
                if y >= 0.0 && y < bucket_number as f32 {
                    let y = y as usize;
                    boxes_y[y] = boxes_y[y].merge(bbox);
                    prim_indices_y[y].push(i);
                }

                let z = (centroid.z - bbox.p_min.z) / len_per_bucket.z;
                if z >= 0.0 && z < bucket_number as f32 {
                    let z = z as usize;
                    boxes_z[z] = boxes_z[z].merge(bbox);
                    prim_indices_z[z].push(i);
                }
            }

            let (best_cost_x, best_split_x) = if len_per_bucket.x > 0.0001 {
                Self::find_best_split(&boxes_x, &prim_indices_x, u.size(), bucket_number)
            } else {
                (f32::MAX, bucket_number / 2)
            };
            let (best_cost_y, best_split_y) = if len_per_bucket.y > 0.0001 {
                Self::find_best_split(&boxes_y, &prim_indices_y, u.size(), bucket_number)
            } else {
                (f32::MAX, bucket_number / 2)
            };
            let (best_cost_z, best_split_z) = if len_per_bucket.z > 0.0001 {
                Self::find_best_split(&boxes_z, &prim_indices_z, u.size(), bucket_number)
            } else {
                (f32::MAX, bucket_number / 2)
            };

            let (lc, rc) = if best_cost_x <= best_cost_y && best_cost_x <= best_cost_z {
                Self::split_at(
                    best_split_x,
                    bucket_number,
                    &boxes_x,
                    &mut prim_indices_x,
                    &mut primitives,
                    u.start,
                    u.end,
                )
            } else if best_cost_y <= best_cost_x && best_cost_y <= best_cost_z {
                Self::split_at(
                    best_split_y,
                    bucket_number,
                    &boxes_y,
                    &mut prim_indices_y,
                    &mut primitives,
                    u.start,
                    u.end,
                )
            } else {
                Self::split_at(
                    best_split_z,
                    bucket_number,
                    &boxes_z,
                    &mut prim_indices_z,
                    &mut primitives,
                    u.start,
                    u.end,
                )
            };
            if lc.size() == 0 || rc.size() == 0 {
                continue;
            }
            u.lc = Some(lc);
            u.rc = Some(rc);

            stack.push(u.lc.as_mut().unwrap());
            stack.push(u.rc.as_mut().unwrap());
        }

        Self {
            bvh_root: Some(bvh_root),
            primitives,
        }
    }

    fn find_best_split(
        boxes: &Vec<Bbox>,
        prim_indices: &Vec<Vec<usize>>,
        prim_total_count: usize,
        bucket_number: usize,
    ) -> (f32, usize) {
        let mut best_cost = f32::MAX;
        let mut best_split = 0;
        let mut boxes_l = boxes.clone();
        let mut boxes_r = boxes.clone();
        let mut prim_count = vec![0; bucket_number];
        prim_count[0] = prim_indices[0].len();
        for i in 1..bucket_number {
            boxes_l[i] = boxes_l[i].merge(boxes_l[i - 1]);
            prim_count[i] = prim_count[i - 1] + prim_indices[i].len();
        }
        for i in (0..bucket_number - 1).rev() {
            boxes_r[i] = boxes_r[i].merge(boxes_r[i + 1]);
        }
        for i in 1..bucket_number {
            let left_surface_area = boxes_l[i - 1].surface_area();
            let left_count = prim_count[i - 1] as f32;
            let right_surface_area = boxes_r[i].surface_area();
            let right_count = prim_total_count as f32 - left_count;
            let cost = left_surface_area * left_count + right_surface_area * right_count;
            if cost < best_cost {
                best_cost = cost;
                best_split = i;
            }
        }
        (best_cost, best_split)
    }

    fn split_at(
        split: usize,
        bucket_number: usize,
        boxes: &Vec<Bbox>,
        prim_indices: &mut Vec<Vec<usize>>,
        primitives: &mut Vec<Arc<P>>,
        start: usize,
        end: usize,
    ) -> (Box<BvhNode>, Box<BvhNode>) {
        let mut left_bbox = Bbox::empty();
        let mut left_indices = vec![];
        for i in 0..split {
            left_bbox = left_bbox.merge(boxes[i]);
            left_indices.append(&mut prim_indices[i]);
        }
        let left_indices: HashSet<usize> = left_indices.into_iter().collect();

        let mut right_bbox = Bbox::empty();
        let mut right_indices = vec![];
        for i in split..bucket_number {
            right_bbox = right_bbox.merge(boxes[i]);
            right_indices.append(&mut prim_indices[i]);
        }
        let right_indices: HashSet<usize> = right_indices.into_iter().collect();

        let mut rp = end - 1;
        let mid = start + left_indices.len();
        for lp in start..mid {
            if !left_indices.contains(&lp) {
                while right_indices.contains(&rp) {
                    rp -= 1;
                }
                primitives.swap(lp, rp);
                rp -= 1;
            }
        }
        let lc = Box::new(BvhNode::new(start, mid, left_bbox));
        let rc = Box::new(BvhNode::new(mid, end, right_bbox));
        (lc, rc)
    }
}

impl BvhNode {
    fn new(start: usize, end: usize, bbox: Bbox) -> Self {
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

impl<P: PrimitiveT> PrimitiveT for BvhAccel<P> {
    fn intersect_test(&self, ray: &Ray, t_max: f32) -> bool {
        if self.bvh_root.is_none() {
            return false;
        }

        let mut stack = vec![self.bvh_root.as_ref().unwrap()];
        while let Some(u) = stack.pop() {
            if !u.bbox.intersect_test(ray, t_max) {
                continue;
            }
            if u.is_leaf() {
                for i in u.start..u.end {
                    if self.primitives[i].intersect_test(ray, t_max) {
                        return true;
                    }
                }
            } else {
                stack.push(u.lc.as_ref().unwrap());
                stack.push(u.rc.as_ref().unwrap());
            }
        }
        false
    }

    fn intersect<'a>(&'a self, ray: &Ray, inter: &mut Intersection<'a>) -> bool {
        if self.bvh_root.is_none() {
            return false;
        }

        let mut stack = vec![self.bvh_root.as_ref().unwrap()];
        let mut result = false;
        while let Some(u) = stack.pop() {
            if !u.bbox.intersect_test(ray, inter.t) {
                continue;
            }
            if u.is_leaf() {
                for i in u.start..u.end {
                    result |= self.primitives[i].intersect(ray, inter);
                }
            } else {
                stack.push(u.lc.as_ref().unwrap());
                stack.push(u.rc.as_ref().unwrap());
            }
        }
        result
    }

    fn bbox(&self) -> Bbox {
        if let Some(root) = &self.bvh_root {
            root.bbox
        } else {
            Bbox::empty()
        }
    }

    fn sample<'a>(&'a self, sampler: &mut Rng) -> (Intersection<'a>, f32) {
        let index = sampler.uniform_1d() * self.primitives.len() as f32;
        let index = (index as usize).min(self.primitives.len() - 1);
        let (inter, pdf) = self.primitives[index].sample(sampler);
        (inter, pdf / self.primitives.len() as f32)
    }

    fn pdf(&self, inter: &Intersection<'_>) -> f32 {
        inter.primitive.unwrap().pdf(inter) / self.primitives.len() as f32
    }
}
