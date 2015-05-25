use na::Pnt3;
use entities::shape::Triangle3;
use procedural::{TriMesh, TriMesh3};
use super::ToTriMesh;
use math::Scalar;

impl<N: Scalar> ToTriMesh<Pnt3<N>, ()> for Triangle3<N> {
    fn to_trimesh(&self, _: ()) -> TriMesh3<N> {
        TriMesh::new(
            vec!(self.a().clone(), self.b().clone(), self.c().clone()),
            None,
            None,
            None)
    }
}
