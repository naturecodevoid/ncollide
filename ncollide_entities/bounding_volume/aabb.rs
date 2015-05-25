//! Axis Aligned Bounding Box.

use std::ops::Neg;
use na::{Translation, POrd, Translate, Bounded};
use na;
use bounding_volume::BoundingVolume;
use math::{Scalar, Point, Vect};

/// Trait of objects that can be bounded by an AABB.
pub trait HasAABB<P, M> {
    /// The object’s AABB.
    fn aabb(&self, &M) -> AABB<P>;
}

/// An Axis Aligned Bounding Box.
#[derive(Debug, PartialEq, Clone, RustcEncodable, RustcDecodable)]
pub struct AABB<P> {
    mins: P,
    maxs: P
}

impl<P: POrd> AABB<P> {
    /// Creates a new AABB.
    ///
    /// # Arguments:
    ///   * `mins` - position of the point with the smallest coordinates.
    ///   * `maxs` - position of the point with the highest coordinates. Each component of `mins`
    ///   must be smaller than the related components of `maxs`.
    pub fn new(mins: P, maxs: P) -> AABB<P> {
        assert!(na::partial_le(&mins, &maxs));

        AABB {
            mins: mins,
            maxs: maxs
        }
    }
}

impl<P: Neg<Output = P> + POrd + Bounded> AABB<P> {
    /// Creates an invalid AABB with:
    /// * `mins = Bounded::max_value()`
    /// * `maxs = Bounded::max_value()`.
    /// This is useful to build aabb using merges.
    pub fn new_invalid() -> AABB<P> {
        let _max: P = Bounded::max_value();
        AABB {
            mins: Bounded::max_value(),
            maxs: -_max,
        }
    }
}

impl<P> AABB<P> {
    /// Reference to the AABB point with the smallest components along each axis.
    #[inline]
    pub fn mins(&self) -> &P {
        &self.mins
    }

    /// Reference to the AABB point with the biggest components along each axis.
    #[inline]
    pub fn maxs(&self) -> &P {
        &self.maxs
    }
}

impl<P> AABB<P>
    where P: Point {
    /// The center of this AABB.
    #[inline]
    pub fn center(&self) -> P {
        na::center(&self.mins, &self.maxs)
    }

    /// The half extents of this AABB.
    #[inline]
    pub fn half_extents(&self) -> P::Vect {
        (self.maxs - self.mins) / na::cast(2.0f64)
    }
}

impl<P> BoundingVolume<<P::Vect as Vect>::Scalar> for AABB<P>
    where P: Point {
    #[inline]
    fn intersects(&self, other: &AABB<P>) -> bool {
        na::partial_le(&self.mins, &other.maxs) &&
        na::partial_ge(&self.maxs, &other.mins)
    }

    #[inline]
    fn contains(&self, other: &AABB<P>) -> bool {
        na::partial_le(&self.mins, &other.mins) &&
        na::partial_ge(&self.maxs, &other.maxs)
    }

    #[inline]
    fn merge(&mut self, other: &AABB<P>) {
        self.mins = na::inf(&self.mins, &other.mins);
        self.maxs = na::sup(&self.maxs, &other.maxs);
    }

    #[inline]
    fn merged(&self, other: &AABB<P>) -> AABB<P> {
        AABB {
            mins: na::inf(&self.mins, &other.mins),
            maxs: na::sup(&self.maxs, &other.maxs)
        }
    }

    #[inline]
    fn loosen(&mut self, amount: <P::Vect as Vect>::Scalar) {
        assert!(amount >= na::zero(), "The loosening margin must be positive.");
        self.mins = self.mins + na::repeat(-amount);
        self.maxs = self.maxs + na::repeat(amount);
    }

    #[inline]
    fn loosened(&self, amount: <P::Vect as Vect>::Scalar) -> AABB<P> {
        assert!(amount >= na::zero(), "The loosening margin must be positive.");
        AABB {
            mins: self.mins + na::repeat(-amount),
            maxs: self.maxs + na::repeat(amount)
        }
    }

    #[inline]
    fn tighten(&mut self, amount: <P::Vect as Vect>::Scalar) {
        assert!(amount >= na::zero(), "The tightening margin must be positive.");
        self.mins = self.mins + na::repeat(amount);
        self.maxs = self.maxs + na::repeat(-amount);
        assert!(na::partial_le(&self.mins, &self.maxs), "The tightening margin is to large.");
    }

    #[inline]
    fn tightened(&self, amount: <P::Vect as Vect>::Scalar) -> AABB<P> {
        assert!(amount >= na::zero(), "The tightening margin must be positive.");

        AABB::new(self.mins + na::repeat(amount), self.maxs + na::repeat(-amount))
    }
}

impl<P> Translation<P::Vect> for AABB<P>
    where P: Point,
          P::Vect: Translate<P> {
    #[inline]
    fn translation(&self) -> P::Vect {
        na::center(&self.mins, &self.maxs).to_vec()
    }

    #[inline]
    fn inv_translation(&self) -> P::Vect {
        -self.translation()
    }

    #[inline]
    fn append_translation_mut(&mut self, dv: &P::Vect) {
        self.mins = self.mins + *dv;
        self.maxs = self.maxs + *dv;
    }

    #[inline]
    fn append_translation(&self, dv: &P::Vect) -> AABB<P> {
        AABB::new(self.mins + *dv, self.maxs + *dv)
    }

    #[inline]
    fn prepend_translation_mut(&mut self, dv: &P::Vect) {
        self.append_translation_mut(dv)
    }

    #[inline]
    fn prepend_translation(&self, dv: &P::Vect) -> AABB<P> {
        self.append_translation(dv)
    }

    #[inline]
    fn set_translation(&mut self, v: P::Vect) {
        let center = self.translation();
        let total_translation = center + v;

        self.mins = na::inv_translate(&total_translation, &self.mins);
        self.maxs = na::inv_translate(&total_translation, &self.maxs);
    }
}
