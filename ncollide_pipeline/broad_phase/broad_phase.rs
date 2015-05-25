use queries::ray::Ray;

/// Trait all broad phase must implement.
pub trait BroadPhase<P, BV, T> {
    /// Tells the broad phase to add an element during the next update.
    fn defered_add(&mut self, uid: usize, bv: BV, data: T);

    /// Tells the broad phase to remove an element during the next update.
    fn defered_remove(&mut self, uid: usize);

    /// Sets the next bounding volume to be used during the update of this broad phase.
    fn defered_set_bounding_volume(&mut self, uid: usize, bv: BV);

    /// Updates the object additions, removals, and interferences detection.
    fn update(&mut self, allow_proximity: &mut FnMut(&T, &T) -> bool, proximity_handler: &mut FnMut(&T, &T, bool));

    /*
     * FIXME: the following are not flexible enough.
     */
    // XXX: return iterators when associated types work.
    /// Collects every object which might intersect a given bounding volume.
    fn interferences_with_bounding_volume<'a>(&'a self, bv: &BV, out: &mut Vec<&'a T>);

    /// Collects every object which might intersect a given ray.
    fn interferences_with_ray<'a>(&'a self, ray: &Ray<P>, out: &mut Vec<&'a T>);

    /// Collects every object which might contain a given point.
    fn interferences_with_point<'a>(&'a self, point: &P, out: &mut Vec<&'a T>);
}
