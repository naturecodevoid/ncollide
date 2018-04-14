use std::ops::Index;
use num::Zero;

use alga::general::Id;
use alga::linear::NormedSpace;
use na::{self, Point2, Vector3};

use query::{ray_internal, Ray, RayCast, RayIntersection};
use shape::{BaseMesh, BaseMeshElement, Polyline, TriMesh};
use bounding_volume::AABB;
use partitioning::BVTCostFn;
use math::{Isometry, Point};

impl<P, M, I, E> RayCast<P, M> for BaseMesh<P, I, E>
where
    N: Real,
    M: Isometry<P>,
    I: Index<usize, Output = usize>,
    E: BaseMeshElement<I, P> + RayCast<P, Id>,
{
    #[inline]
    fn toi_with_ray(&self, m: &Isometry<N>, ray: &Ray<P>, _: bool) -> Option<N> {
        let ls_ray = ray.inverse_transform_by(m);

        let mut cost_fn = BaseMeshRayToiCostFn {
            mesh: self,
            ray: &ls_ray,
        };

        self.bvt()
            .best_first_search(&mut cost_fn)
            .map(|(_, res)| res)
    }

    #[inline]
    fn toi_and_normal_with_ray(
        &self,
        m: &Isometry<N>,
        ray: &Ray<P>,
        _: bool,
    ) -> Option<RayIntersection<Vector<N>>> {
        let ls_ray = ray.inverse_transform_by(m);

        let mut cost_fn = BaseMeshRayToiAndNormalCostFn {
            mesh: self,
            ray: &ls_ray,
        };

        self.bvt()
            .best_first_search(&mut cost_fn)
            .map(|(_, mut res)| {
                res.normal = m.rotate_vector(&res.normal);
                res
            })
    }

    fn toi_and_normal_and_uv_with_ray(
        &self,
        m: &Isometry<N>,
        ray: &Ray<P>,
        solid: bool,
    ) -> Option<RayIntersection<Vector<N>>> {
        if self.uvs().is_none() || na::dimension::<Vector<N>>() != 3 {
            return self.toi_and_normal_with_ray(m, ray, solid);
        }

        let ls_ray = ray.inverse_transform_by(m);

        let mut cost_fn = BaseMeshRayToiAndNormalAndUVsCostFn {
            mesh: self,
            ray: &ls_ray,
        };
        let cast = self.bvt().best_first_search(&mut cost_fn);

        match cast {
            None => None,
            Some((best, inter)) => {
                let toi = inter.0.toi;
                let n = inter.0.normal;
                let uv = inter.1; // barycentric coordinates to compute the exact uvs.

                let idx = &self.indices()[*best];
                let uvs = self.uvs().as_ref().unwrap();

                let uv1 = uvs[idx[0]];
                let uv2 = uvs[idx[1]];
                let uv3 = uvs[idx[2]];

                let uvx = uv1.x * uv.x + uv2.x * uv.y + uv3.x * uv.z;
                let uvy = uv1.y * uv.x + uv2.y * uv.y + uv3.y * uv.z;

                // XXX: this interpolation should be done on the two other ray cast too!
                match *self.normals() {
                    None => Some(RayIntersection::new_with_uvs(
                        toi,
                        m.rotate_vector(&n),
                        Some(Point2::new(uvx, uvy)),
                    )),
                    Some(ref ns) => {
                        let n1 = &ns[idx[0]];
                        let n2 = &ns[idx[1]];
                        let n3 = &ns[idx[2]];

                        let mut n123 = *n1 * uv.x + *n2 * uv.y + *n3 * uv.z;

                        if n123.normalize_mut().is_zero() {
                            Some(RayIntersection::new_with_uvs(
                                toi,
                                m.rotate_vector(&n),
                                Some(Point2::new(uvx, uvy)),
                            ))
                        } else {
                            if na::dot(&n123, &ls_ray.dir) > na::zero() {
                                Some(RayIntersection::new_with_uvs(
                                    toi,
                                    -m.rotate_vector(&n123),
                                    Some(Point2::new(uvx, uvy)),
                                ))
                            } else {
                                Some(RayIntersection::new_with_uvs(
                                    toi,
                                    m.rotate_vector(&n123),
                                    Some(Point2::new(uvx, uvy)),
                                ))
                            }
                        }
                    }
                }
            }
        }
    }
}

/*
 * Costs functions.
 */
struct BaseMeshRayToiCostFn<'a, P: 'a + Point, I: 'a, E: 'a> {
    mesh: &'a BaseMesh<P, I, E>,
    ray: &'a Ray<P>,
}

impl<'a, P, I, E> BVTCostFn<N, usize, AABB<N>> for BaseMeshRayToiCostFn<'a, P, I, E>
where
    N: Real,
    E: BaseMeshElement<I, P> + RayCast<P, Id>,
{
    type UserData = N;

    #[inline]
    fn compute_bv_cost(&mut self, aabb: &AABB<N>) -> Option<N> {
        aabb.toi_with_ray(&Id::new(), self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &usize) -> Option<(N, N)> {
        self.mesh
            .element_at(*b)
            .toi_with_ray(&Id::new(), self.ray, true)
            .map(|toi| (toi, toi))
    }
}

struct BaseMeshRayToiAndNormalCostFn<'a, P: 'a + Point, I: 'a, E: 'a> {
    mesh: &'a BaseMesh<P, I, E>,
    ray: &'a Ray<P>,
}

impl<'a, P, I, E> BVTCostFn<N, usize, AABB<N>> for BaseMeshRayToiAndNormalCostFn<'a, P, I, E>
where
    N: Real,
    E: BaseMeshElement<I, P> + RayCast<P, Id>,
{
    type UserData = RayIntersection<Vector<N>>;

    #[inline]
    fn compute_bv_cost(&mut self, aabb: &AABB<N>) -> Option<N> {
        aabb.toi_with_ray(&Id::new(), self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &usize) -> Option<(N, RayIntersection<Vector<N>>)> {
        self.mesh
            .element_at(*b)
            .toi_and_normal_with_ray(&Id::new(), self.ray, true)
            .map(|inter| (inter.toi, inter))
    }
}

struct BaseMeshRayToiAndNormalAndUVsCostFn<'a, P: 'a + Point, I: 'a, E: 'a> {
    mesh: &'a BaseMesh<P, I, E>,
    ray: &'a Ray<P>,
}

impl<'a, P, I, E> BVTCostFn<N, usize, AABB<N>>
    for BaseMeshRayToiAndNormalAndUVsCostFn<'a, P, I, E>
where
    N: Real,
    I: Index<usize, Output = usize>,
    E: BaseMeshElement<I, P> + RayCast<P, Id>,
{
    type UserData = (RayIntersection<Vector<N>>, Vector3<N>);

    #[inline]
    fn compute_bv_cost(&mut self, aabb: &AABB<N>) -> Option<N> {
        aabb.toi_with_ray(&Id::new(), self.ray, true)
    }

    #[inline]
    fn compute_b_cost(
        &mut self,
        b: &usize,
    ) -> Option<(N, (RayIntersection<Vector<N>>, Vector3<N>))> {
        let vs = &self.mesh.vertices()[..];
        let idx = &self.mesh.indices()[*b];

        let a = &vs[idx[0]];
        let b = &vs[idx[1]];
        let c = &vs[idx[2]];

        ray_internal::triangle_ray_intersection(a, b, c, self.ray).map(|inter| (inter.0.toi, inter))
    }
}

/*
 * fwd impls. to the exact shapes.
 */
impl<N: Real> RayCast<P, M> for TriMesh<P> {
    #[inline]
    fn toi_with_ray(&self, m: &Isometry<N>, ray: &Ray<P>, solid: bool) -> Option<N> {
        self.base_mesh().toi_with_ray(m, ray, solid)
    }

    #[inline]
    fn toi_and_normal_with_ray(
        &self,
        m: &Isometry<N>,
        ray: &Ray<P>,
        solid: bool,
    ) -> Option<RayIntersection<Vector<N>>> {
        self.base_mesh().toi_and_normal_with_ray(m, ray, solid)
    }

    #[inline]
    fn toi_and_normal_and_uv_with_ray(
        &self,
        m: &Isometry<N>,
        ray: &Ray<P>,
        solid: bool,
    ) -> Option<RayIntersection<Vector<N>>> {
        self.base_mesh()
            .toi_and_normal_and_uv_with_ray(m, ray, solid)
    }
}

impl<N: Real> RayCast<P, M> for Polyline<P> {
    #[inline]
    fn toi_with_ray(&self, m: &Isometry<N>, ray: &Ray<P>, solid: bool) -> Option<N> {
        self.base_mesh().toi_with_ray(m, ray, solid)
    }

    #[inline]
    fn toi_and_normal_with_ray(
        &self,
        m: &Isometry<N>,
        ray: &Ray<P>,
        solid: bool,
    ) -> Option<RayIntersection<Vector<N>>> {
        self.base_mesh().toi_and_normal_with_ray(m, ray, solid)
    }

    #[inline]
    fn toi_and_normal_and_uv_with_ray(
        &self,
        m: &Isometry<N>,
        ray: &Ray<P>,
        solid: bool,
    ) -> Option<RayIntersection<Vector<N>>> {
        self.base_mesh()
            .toi_and_normal_and_uv_with_ray(m, ray, solid)
    }
}
