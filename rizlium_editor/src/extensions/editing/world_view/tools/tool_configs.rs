use bevy::prelude::*;
use rizlium_chart::chart::EasingId;
use std::marker::PhantomData;

use egui::{Slider, Ui, UiBuilder};
use rizlium_render::GameChart;

use crate::widgets::enum_selector;

pub(crate) fn show_window<T: ToolConfig>(ui: &mut Ui, world: &mut World) {
    let child = {
        let max_rect = ui.available_rect_before_wrap();
        let layout = *ui.layout();
        ui.new_child(
            UiBuilder::new()
                .max_rect(max_rect)
                .layout(layout)
        )
    };
    world.resource_scope(|world, mut stroage: Mut<'_, ToolConfigStorage<T>>| {
        stroage.0.run(child, world);
    });
}

#[derive(Resource)]
pub struct ToolConfigStorage<T: ToolConfig>(Box<dyn System<In = Ui, Out = ()>>, PhantomData<T>);

impl<T: ToolConfig> ToolConfigStorage<T> {
    pub(crate) fn init_with(mut self, world: &mut World) -> Self {
        self.0.initialize(world);
        self
    }
}

impl<T: ToolConfig> Default for ToolConfigStorage<T> {
    fn default() -> Self {
        Self(Box::new(T::config_system()), PhantomData)
    }
}

pub trait ToolConfigExt {
    fn init_tool_config<T>(&mut self) -> &mut Self
    where
        T: ToolConfig + Resource + Default;
}

impl ToolConfigExt for App {
    fn init_tool_config<T>(&mut self) -> &mut Self
    where
        T: ToolConfig + Resource + Default,
    {
        let resource = ToolConfigStorage::<T>::default().init_with(self.world_mut());
        self.init_resource::<T>().insert_resource(resource)
    }
}

pub trait ToolConfig: Send + Sync + 'static {
    fn config_system() -> impl System<In = Ui, Out = ()>;
}

impl ToolConfig for PencilToolConfig {
    fn config_system() -> impl System<In = Ui, Out = ()> {
        IntoSystem::into_system(Self::system)
    }
}

#[derive(Resource, Default)]
pub struct PencilToolConfig {
    pub canvas: usize,
    pub pen_color: egui::Color32,
    pub easing: EasingId,
}

impl PencilToolConfig {
    pub(crate) fn system(In(mut ui): In<Ui>, mut this: ResMut<Self>, chart: Res<GameChart>) {
        ui.columns(2, |uis| {
            let [uil, uir] = uis else {
                // must be two
                return;
            };
            uil.label("Canvas index:");
            uir.add(Slider::new(
                &mut this.canvas,
                0..=(chart.canvases.len() - 1),
            ));
            uil.label("Color: ");
            uir.color_edit_button_srgba(&mut this.pen_color);
            uil.label("Easing");
            enum_selector(&mut this.easing, uir);
        })
    }
}
