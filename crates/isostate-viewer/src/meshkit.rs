//! The kit for building low-poly, flat-shaded, vertex-coloured meshes. The
//! buffer of triangles and its shapes live in the generator crate
//! (`isostate_gen::MeshBuf`), which has no Bevy in it; `Builder` is the
//! viewer's older name for the same thing. Everything of one kind (all trees,
//! all houses) is merged into one buffer, rebuilt when the year changes, and
//! this is the one place a buffer becomes a Bevy mesh.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};

pub use isostate_gen::MeshBuf as Builder;
pub use isostate_gen::{hex, lerp, shade};

pub fn into_mesh(b: Builder) -> Mesh {
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, b.pos);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, b.nrm);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, b.col);
    m.insert_indices(Indices::U32(b.idx));
    m
}
