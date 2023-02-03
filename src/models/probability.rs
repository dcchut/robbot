use probability::distribution::{Distribution, Gaussian};

#[derive(Clone, Debug, Default)]
pub struct GaussianDist {
    dist: Gaussian,
}

impl GaussianDist {
    pub fn new() -> Self {
        Self {
            dist: Gaussian::new(0., 1.),
        }
    }

    pub fn normal_cdf(&self, lower: f64, upper: f64) -> String {
        let p = self.dist.distribution(upper) - self.dist.distribution(lower);
        format!("P({lower} <= Z <= {upper}) = {p}")
    }

    pub fn normal_cdf_one_sided(&self, upper: f64) -> String {
        format!("P(Z <= {upper}) = {}", self.dist.distribution(upper))
    }
}
