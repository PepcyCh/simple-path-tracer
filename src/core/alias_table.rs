pub struct AliasTable {
    props: Vec<f32>,
    u: Vec<f32>,
    k: Vec<usize>,
}

impl AliasTable {
    pub fn new(props: Vec<f32>) -> Self {
        let n = props.len();
        let mut u: Vec<f32> = props.iter().map(|prop| *prop * n as f32).collect();
        let mut k: Vec<usize> = (0..n).collect();

        let mut poor = u
            .iter()
            .enumerate()
            .find(|(_, val)| **val < 1.0)
            .map(|(ind, _)| ind);
        let mut poor_max = poor;
        let mut rich = u
            .iter()
            .enumerate()
            .find(|(_, val)| **val > 1.0)
            .map(|(ind, _)| ind);

        while poor.is_some() && rich.is_some() {
            let poor_ind = poor.unwrap();
            let rich_ind = rich.unwrap();

            let diff = 1.0 - u[poor_ind];
            u[rich_ind] -= diff;
            k[poor_ind] = rich_ind;

            if u[rich_ind] < 1.0 && rich_ind < poor_max.unwrap() {
                poor = Some(rich_ind);
            } else {
                poor = None;
                for i in poor_max.unwrap() + 1..u.len() {
                    if u[i] < 1.0 {
                        poor = Some(i);
                        poor_max = Some(i);
                        break;
                    }
                }
            }

            rich = None;
            for i in rich_ind..u.len() {
                if u[i] > 1.0 {
                    rich = Some(i);
                    break;
                }
            }
        }

        Self { props, u, k }
    }

    pub fn sample(&self, rand: f32) -> (usize, f32) {
        let temp = rand * self.props.len() as f32;
        let x = temp as usize;
        let y = temp - x as f32;
        if y < self.u[x] {
            (x, self.props[x])
        } else {
            (self.k[x], self.props[self.k[x]])
        }
    }

    pub fn probability(&self, index: usize) -> f32 {
        self.props[index]
    }
}
