use bevy::{
    ecs::{
        resource::Resource,
        system::Commands,
        world::{FromWorld, World},
    },
    render::{
        render_resource::{Buffer, BufferInitDescriptor, BufferSlice, BufferUsages},
        renderer::RenderDevice,
    },
};
use core::ops::RangeBounds;

/// Static index buffer shared by every cuboid draw.
///
/// The contents are compile-time constant, so this is a plain render-world resource rather than a
/// [`RenderAsset`](bevy::render::render_asset::RenderAsset): there is no source asset to track and
/// nothing to re-upload.
#[derive(Resource)]
pub struct CuboidsIndexBuffer(Buffer);

impl CuboidsIndexBuffer {
    pub(crate) fn slice(&self, bounds: impl RangeBounds<u64>) -> BufferSlice<'_> {
        self.0.slice(bounds)
    }
}

// Only 3 faces are actually drawn.
const NUM_CUBE_INDICES_USIZE: usize = 3 * 3 * 2;

/// The indices for all triangles in a cuboid mesh (given 8 corner
/// vertices).
///
/// In addition to encoding the 3-bit cube corner index, we add 2 bits
/// to indicate which of the 3 faces is being rendered.
#[rustfmt::skip]
#[allow(clippy::unusual_byte_groupings)]
pub(crate) const CUBE_INDICES: [u32; NUM_CUBE_INDICES_USIZE] = [
    0b00_000, 0b00_010, 0b00_001, 0b00_010, 0b00_011, 0b00_001, // face XY (0)
    0b01_101, 0b01_100, 0b01_001, 0b01_001, 0b01_100, 0b01_000, // face XZ (1)
    0b10_000, 0b10_100, 0b10_110, 0b10_000, 0b10_110, 0b10_010, // face YZ (2)
];

pub(crate) fn prepare_cuboids_index_buffer(mut commands: Commands) {
    commands.init_resource::<CuboidsIndexBuffer>()
}

impl FromWorld for CuboidsIndexBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::INDEX,
            label: Some("Cuboid Index Buffer"),
            contents: bytemuck::cast_slice(CUBE_INDICES.as_slice()),
        });
        Self(buffer)
    }
}
