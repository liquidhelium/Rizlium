use std::{
    io::{Read, Write}, ops::Deref, process::{ChildStderr, ChildStdin, Stdio}
};

use bevy::{
    app::ScheduleRunnerPlugin,
    ecs::system::lifetimeless::SRes,
    prelude::*,
    render::{
        graph::CameraDriverLabel,
        render_asset::{RenderAsset, RenderAssetPlugin, RenderAssetUsages, RenderAssets},
        render_graph::{Node, RenderGraph, RenderLabel},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, Extent3d, ImageCopyBuffer, ImageDataLayout,
            Maintain, MapMode, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
        Render, RenderApp, RenderSet,
    },
    winit::WinitPlugin,
};
use ffmpeg_sidecar::command::FfmpegCommand;
use futures::channel::oneshot;
use rizlium_render::{GameView, TimeManager};

fn main() {
    let mut ffmpeg = FfmpegCommand::new();
    ffmpeg
        .overwrite()
        .format("rawvideo")
        .hwaccel("auto")
        .size(1080, 1920)
        .rate(60.0)
        .pix_fmt("bgra")
        .input("-")
        .duration("10s")
        .output("video.mp4")
        .codec_video("libx264")
        .print_command();
    ffmpeg.as_inner_mut().stdin(Stdio::piped());
    let mut ffmpeg = ffmpeg.spawn().unwrap();

    let stdin = ffmpeg.take_stdin().unwrap();
    let stderr = ffmpeg.take_stderr().unwrap();
    let mut app = App::new();
    let app_mut = app
        .add_plugins((
            DefaultPlugins.build().disable::<WinitPlugin>(),
            ScheduleRunnerPlugin::default(),
        ))
        .register_type::<VideoTexture>()
        .init_asset::<VideoTexture>()
        .register_asset_reflect::<VideoTexture>()
        .add_plugins(rizlium_render::RizliumRenderingPlugin {
            config: (),
            init_with_chart: Some(rizlium_render::rizlium_chart::test_resources::CHART.deref().clone())
        })
        .add_plugins(RenderAssetPlugin::<VideoTexture>::default())
        .add_systems(PostStartup, setup_game_view);
    let render = app_mut.sub_app_mut(RenderApp);
    render
        .add_systems(
            Render,
            test.after(RenderSet::Render).before(RenderSet::Cleanup),
        )
        .insert_resource(FfmpegIn((stdin, stderr, 0)));
    let mut graph = render.world.get_resource_mut::<RenderGraph>().unwrap();
    graph.add_node(VideoNodeLabel, VideoRenderNode);
    graph.add_node_edge(CameraDriverLabel, VideoNodeLabel);
    for _ in 0..100 {
        app.run();
    }
    drop(app);
    ffmpeg.wait().unwrap();
}

#[derive(Resource, Default)]
struct VideoTextureStorage(Handle<VideoTexture>);

fn setup_game_view(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut textured: ResMut<Assets<VideoTexture>>,
    mut time: ResMut<TimeManager>
) {
    let size = Extent3d {
        width: 1080,
        height: 1920,
        ..default()
    };
    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    let image_handle = images.add(image);
    let handle = textured.add(VideoTexture(image_handle.clone()));
    commands.insert_resource(VideoTextureStorage(handle));
    commands.insert_resource(GameView(image_handle));
    time.set_paused(false);
}

#[derive(Resource)]
pub struct FfmpegIn((ChildStdin, ChildStderr, usize));

fn test(
    sources: Res<RenderAssets<VideoTexture>>,
    render_device: Res<RenderDevice>,
    mut ffmpeg: ResMut<FfmpegIn>,
) {
    for (_, asset) in sources.iter() {
        let mut bytes = {
            let slice = asset.buffer.slice(..);
            {
                let (mapping_tx, mapping_rx) = oneshot::channel();
                // 这才是映射gpu的data到内存里的地方
                render_device.map_buffer(&slice, MapMode::Read, move |res| {
                    mapping_tx.send(res).unwrap();
                });

                render_device.poll(Maintain::Wait);
                futures_lite::future::block_on(mapping_rx).unwrap().unwrap();
            }
            slice.get_mapped_range().to_vec()
        };
        // 每次用完还得还
        asset.buffer.unmap();
        let bytes_per_row = asset.bytes_per_row as usize;
        let padded_bytes_per_row = asset.padded_bytes_per_row as usize;
        let source_size = asset.source_size;
        if bytes_per_row != padded_bytes_per_row {
            let mut unpadded_bytes =
                Vec::<u8>::with_capacity(source_size.height as usize * bytes_per_row);

            for padded_row in bytes.chunks(padded_bytes_per_row) {
                unpadded_bytes.extend_from_slice(&padded_row[..bytes_per_row]);
            }

            bytes = unpadded_bytes;
        }
        let inside = &mut ffmpeg.0;
        info!("writing {}", inside.2);
        inside.0.write_all(&bytes).map_err(|err| error!("{err}")).ok();
        inside.0.flush().map_err(|err| error!("{err}")).ok();
        info!("frame {} finished", inside.2);
        inside.2 += 1;
    }
}

#[derive(Asset, Clone, Default, Reflect)]
pub struct VideoTexture(pub Handle<Image>);

pub struct GpuVideoTexture {
    pub buffer: Buffer,
    pub source_handle: Handle<Image>,
    pub source_size: Extent3d,
    pub bytes_per_row: u32,
    pub padded_bytes_per_row: u32,
    pub format: TextureFormat,
}

impl RenderAsset for VideoTexture {
    type Param = (SRes<RenderDevice>, SRes<RenderAssets<Image>>);
    type PreparedAsset = GpuVideoTexture;
    fn asset_usage(&self) -> bevy::render::render_asset::RenderAssetUsages {
        RenderAssetUsages::RENDER_WORLD
    }
    fn prepare_asset(
        self,
        (device, images): &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, bevy::render::render_asset::PrepareAssetError<Self>> {
        info!("prepare asset");
        let gpu_image = images.get(&self.0).unwrap();

        let size = gpu_image.texture.size();
        let format = &gpu_image.texture_format;
        let bytes_per_row =
            (size.width / format.block_dimensions().0) * format.block_copy_size(None).unwrap();
        let padded_bytes_per_row =
            RenderDevice::align_copy_bytes_per_row(bytes_per_row as usize) as u32;

        let source_size = gpu_image.texture.size();

        Ok(GpuVideoTexture {
            buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Image Export Buffer"),
                size: (source_size.height * padded_bytes_per_row) as u64,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            source_handle: self.0.clone(),
            source_size,
            bytes_per_row,
            padded_bytes_per_row,
            format: gpu_image.texture_format,
        })
    }
}

#[derive(RenderLabel, PartialEq, Eq, Hash, Debug, Clone)]
pub struct VideoNodeLabel;

pub struct VideoRenderNode;

impl Node for VideoRenderNode {
    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        for (_, source) in world.resource::<RenderAssets<VideoTexture>>().iter() {
            if let Some(gpu_image) = world
                .resource::<RenderAssets<Image>>()
                .get(&source.source_handle)
            {
                // 这之后, data已经到了buffer里, 只是还在gpu上.
                render_context.command_encoder().copy_texture_to_buffer(
                    gpu_image.texture.as_image_copy(),
                    ImageCopyBuffer {
                        buffer: &source.buffer,
                        layout: ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(source.padded_bytes_per_row),
                            rows_per_image: None,
                        },
                    },
                    source.source_size,
                );
            }
        }

        Ok(())
    }
}
