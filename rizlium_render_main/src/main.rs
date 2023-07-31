use bevy::{prelude::*, render::render_resource::TextureDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy_egui::EguiPlugin;
use bevy_egui::{EguiContexts, egui::{self, FontDefinitions, FontData}};
use rizlium_render::{GameView, RizliumRenderingPlugin, TimeManager};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            RizliumRenderingPlugin {
                config: (),
                init_with_chart: Some(rizlium_chart::__test_chart())
            },
        ))
        .add_systems(PreStartup, setup_game_view.after(bevy_egui::EguiStartupSet::InitContexts))
        .add_systems(Update, egui_render)
        .run()
        ;

}
fn setup_game_view(
    mut commands: Commands,
    mut egui_context: EguiContexts,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 1080,
        height: 1920,
        ..default()
    };
    let data = font_kit::source::SystemSource::new()
        .select_by_postscript_name("SourceHanSansCN-Normal")
        .unwrap()
        .load()
        .unwrap()
        .copy_font_data()
        .unwrap();
    let mut def = FontDefinitions::default();
    def.font_data
        .insert("cn".into(), FontData::from_owned(Vec::clone(&data)));
    def.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "cn".to_owned());
    let ctx = egui_context.ctx_mut();
    ctx.set_fonts(def);
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
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    let image_handle = images.add(image);
    egui_context.add_image(image_handle.clone());
    commands.insert_resource(GameView(image_handle.clone()));
    
}
fn egui_render(
    mut egui_context: EguiContexts,
    gameview: Res<GameView>,
    mut time: ResMut<TimeManager>,
) {
    let img = egui_context
        .image_id(&gameview.0)
        .expect("no gameview image found!");
    let ctx = egui_context.ctx_mut();
    egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        ui.label("Rizlium");
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.image(img, [450., 600.]);
        if ui.button("pause").clicked() {
            time.toggle_paused();
        }
    });
}