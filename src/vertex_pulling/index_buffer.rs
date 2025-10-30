use bevy::{
    asset::{Asset, Handle, RenderAssetUsages},
    ecs::{
        component::Component,
        resource::Resource,
        system::{lifetimeless::SRes, Commands, SystemParamItem},
        world::{FromWorld, World},
    },
    log::info,
    reflect::TypePath,
    render::{
        extract_component::ExtractComponent,
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{Buffer, BufferInitDescriptor, BufferUsages},
        renderer::{RenderDevice, RenderQueue},
    },
};

#[derive(Resource)]
// #[uuid = "8f6d78a6-fffe-4e54-81db-08b0739a947a"]
pub struct CuboidsIndexBuffer(Buffer);

// pub(crate) const CUBE_INDICES_HANDLE: Handle<CuboidsIndexBuffer> =
//     Handle::weak_from_u128(17343092250772987267);

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

// impl RenderAsset for CuboidsIndexBuffer {
//     // Nothing, because it's static CPU data
//     type SourceAsset = ();

//     type Param = SRes<RenderDevice>;

//     fn asset_usage(_source_asset: &Self::SourceAsset) -> RenderAssetUsages {
//         RenderAssetUsages::RENDER_WORLD
//     }

//     fn byte_len(_source_asset: &Self::SourceAsset) -> Option<usize> {
//         Some(size_of_val(&CUBE_INDICES))
//     }

//     fn prepare_asset(
//         source_asset: Self::SourceAsset,
//         asset_id: bevy::asset::AssetId<Self::SourceAsset>,
//         render_device: &mut SystemParamItem<Self::Param>,
//         previous_asset: Option<&Self>,
//     ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
//         let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
//             usage: BufferUsages::INDEX,
//             label: Some("Cuboid Index Buffer"),
//             // contents: cast_slice(CUBE_INDICES.as_slice()),
//             contents: unsafe {
//                 std::slice::from_raw_parts(CUBE_INDICES.as_ptr().cast(), size_of_val(&CUBE_INDICES))
//             },
//         });
//         Ok(Self(buffer))
//     }
// }
impl FromWorld for CuboidsIndexBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        // let render_queue = world.resource::<RenderQueue>();
        // // Nothing, because it's static CPU data
        // type SourceAsset = ();

        // type Param = SRes<RenderDevice>;

        // fn asset_usage(_source_asset: &Self::SourceAsset) -> RenderAssetUsages {
        //     RenderAssetUsages::RENDER_WORLD
        // }

        // fn byte_len(_source_asset: &Self::SourceAsset) -> Option<usize> {
        //     Some(size_of_val(&CUBE_INDICES))
        // }

        // fn prepare_asset(
        //     source_asset: Self::SourceAsset,
        //     asset_id: bevy::asset::AssetId<Self::SourceAsset>,
        //     render_device: &mut SystemParamItem<Self::Param>,
        //     previous_asset: Option<&Self>,
        // ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        // let mut buffer = RawBufferVec::new(BufferUsages::INDEX);
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::INDEX,
            label: Some("Cuboid Index Buffer"),
            // contents: cast_slice(CUBE_INDICES.as_slice()),
            contents: unsafe {
                std::slice::from_raw_parts(CUBE_INDICES.as_ptr().cast(), size_of_val(&CUBE_INDICES))
            },
        });
        info!("Init buffer {buffer:?}");
        Self(buffer)
    }
}
