use bevy::{prelude::*, render::render_resource::TextureDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy_egui::EguiPlugin;
use bevy_egui::{EguiContexts, egui::{self, FontDefinitions, FontData, Ui}};
use rizlium_render::{GameView, RizliumRenderingPlugin, TimeManager};

#[derive(Debug, Resource, Default)]
struct EditorState {
    debug_resources: DebugResources,
}

#[derive(Debug, Default)]
struct DebugResources {
    show_cursor: bool
}

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
        .init_resource::<EditorState>()
        .add_systems(PreStartup, (setup_game_view, egui_font).after(bevy_egui::EguiStartupSet::InitContexts))
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

fn egui_font(mut egui_context: EguiContexts) {
    // TODO: this font name is hard coded
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
}
fn egui_render(
    mut egui_context: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    gameview: Res<GameView>,
    mut time: ResMut<TimeManager>,
) {
    let img = egui_context
        .image_id(&gameview.0)
        .expect("no gameview image found!");
    let ctx = egui_context.ctx_mut();
    egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        if editor_state.debug_resources.show_cursor {
            ui.style_mut().debug.debug_on_hover = true;
        }
        ui.label("Rizlium");
        ui.toggle_value(&mut editor_state.debug_resources.show_cursor, "Show cursor (Debug)");
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        if editor_state.debug_resources.show_cursor {
            ui.style_mut().debug.debug_on_hover = true;
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {

            let button = &ui.button("暂停");
            if button.is_pointer_button_down_on() && !button.dragged() {
                time.toggle_paused();
            }
            
            keep_ratio(ui, 16./9., |ui, size| {
                ui.centered_and_justified(|ui| ui.image(img, size));
                
            })
            
        });
    });
}

fn keep_ratio(ui: &mut Ui, ratio: f32,mut add_fn: impl FnMut(&mut Ui, egui::Vec2)) {
    assert_ne!(ratio, 0.);
    let current_size = ui.available_size();
    let mut new_size = egui::Vec2::default();
    if current_size.x < current_size.y/ ratio {
        new_size.x = current_size.x;
        new_size.y = current_size.x*ratio;
    }
    else {
        new_size.x = current_size.y/ratio;
        new_size.y = current_size.y;
    }
    add_fn(ui, new_size);

}