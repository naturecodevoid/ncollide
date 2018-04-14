use std::marker::PhantomData;

use alga::general::Id;

use na;
use bounding_volume::AABB;
use partitioning::BVTCostFn;
use shape::{CompositeShape, Shape};
use query::{self, ClosestPoints, PointQuery};
use math::{Isometry, Point};

/// Closest points between a composite shape and any other shape.
pub fn composite_shape_against_shape<P, M, G1: ?Sized>(
    m1: &Isometry<N>,
    g1: &G1,
    m2: &Isometry<N>,
    g2: &Shape<N>,
    margin: N,
) -> ClosestPoints<P>
where
    N: Real,
    M: Isometry<P>,
    G1: CompositeShape<P, M>,
{
    let mut cost_fn = CompositeShapeAgainstClosestPointsCostFn::new(m1, g1, m2, g2, margin);

    g1.bvt()
        .best_first_search(&mut cost_fn)
        .map(|(_, res)| res)
        .expect("The composite shape must not be empty.")
}

/// Closest points between a shape and a composite shape.
pub fn shape_against_composite_shape<P, M, G2: ?Sized>(
    m1: &Isometry<N>,
    g1: &Shape<N>,
    m2: &Isometry<N>,
    g2: &G2,
    margin: N,
) -> ClosestPoints<P>
where
    N: Real,
    M: Isometry<P>,
    G2: CompositeShape<P, M>,
{
    let mut res = composite_shape_against_shape(m2, g2, m1, g1, margin);
    res.flip();
    res
}

struct CompositeShapeAgainstClosestPointsCostFn<'a, P: 'a + Point, M: 'a, G1: ?Sized + 'a> {
    msum_shift: Vector<N>,
    msum_margin: Vector<N>,
    margin: N,
    stop: bool,

    m1: &'a M,
    g1: &'a G1,
    m2: &'a M,
    g2: &'a Shape<N>,
    point_type: PhantomData<P>,
}

impl<'a, P, M, G1: ?Sized> CompositeShapeAgainstClosestPointsCostFn<'a, P, M, G1>
where
    N: Real,
    M: Isometry<P>,
    G1: CompositeShape<P, M>,
{
    pub fn new(
        m1: &'a M,
        g1: &'a G1,
        m2: &'a M,
        g2: &'a Shape<N>,
        margin: N,
    ) -> CompositeShapeAgainstClosestPointsCostFn<'a, P, M, G1> {
        let ls_m2 = na::inverse(m1) * m2.clone();
        let ls_aabb2 = g2.aabb(&ls_m2);

        CompositeShapeAgainstClosestPointsCostFn {
            msum_shift: -ls_aabb2.center().coords,
            msum_margin: ls_aabb2.half_extents(),
            margin: margin,
            stop: false,
            m1: m1,
            g1: g1,
            m2: m2,
            g2: g2,
            point_type: PhantomData,
        }
    }
}

impl<'a, P, M, G1: ?Sized> BVTCostFn<N, usize, AABB<N>>
    for CompositeShapeAgainstClosestPointsCostFn<'a, P, M, G1>
where
    N: Real,
    M: Isometry<P>,
    G1: CompositeShape<P, M>,
{
    type UserData = ClosestPoints<P>;
    #[inline]
    fn compute_bv_cost(&mut self, bv: &AABB<N>) -> Option<N> {
        // Compute the minkowski sum of the two AABBs.
        let msum = AABB::new(
            *bv.mins() + self.msum_shift + (-self.msum_margin),
            *bv.maxs() + self.msum_shift + self.msum_margin,
        );

        if self.stop {
            None // No need to look further.
        } else {
            // Compute the distance to the origin.
            Some(msum.distance_to_point(&Id::new(), &Point::origin(), true))
        }
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &usize) -> Option<(N, ClosestPoints<P>)> {
        let mut res = None;

        self.g1
            .map_transformed_part_at(*b, self.m1, &mut |_, m1, g1| {
                let pts = query::closest_points(m1, g1, self.m2, self.g2, self.margin);
                let dist = match pts {
                    ClosestPoints::WithinMargin(ref p1, ref p2) => na::distance(p1, p2),
                    ClosestPoints::Intersecting => {
                        self.stop = true;
                        na::zero()
                    }
                    ClosestPoints::Disjoint => self.margin,
                };

                res = Some((dist, pts))
            });

        res
    }
}
