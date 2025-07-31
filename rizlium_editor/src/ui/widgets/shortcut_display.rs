use egui::{Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke};

pub fn shortcut_display(shortcut: &[String], ui: &mut egui::Ui) {
    let spacing = 3.0;
    let key_padding = egui::vec2(6.0, 2.0);
    let key_height = 16.0;
    
    ui.horizontal(|ui| {
        let mut total_width = 0.0;
        let painter = ui.painter();
        
        let plus_width =  painter
                .layout_no_wrap(String::from("+"), FontId::default(), Color32::WHITE)
                .size()
                .x;
        // 计算总宽度
        for (i, key) in shortcut.iter().enumerate() {
            let key_width = painter
                .layout_no_wrap(key.clone(), FontId::monospace(11.0), Color32::WHITE)
                .size()
                .x
                + key_padding.x * 2.0;
            total_width += key_width;
            if i < shortcut.len() - 1 {
                total_width += plus_width + spacing * 2.0;
            }
        }
        
        // 为整个快捷键组合分配空间
        let available_height = key_height;
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(total_width, available_height), Sense::hover());
        
        let painter = ui.painter();
        let mut cursor_x = rect.left();
        let center_y = rect.center().y;

        // 绘制每个按键
        for (i, key) in shortcut.iter().enumerate() {
            let key_width = painter
                .layout_no_wrap(key.clone(), FontId::monospace(11.0), Color32::WHITE)
                .size()
                .x
                + key_padding.x * 2.0;
            let key_rect = Rect::from_min_size(
                egui::pos2(cursor_x, center_y - key_height / 2.0),
                egui::vec2(key_width, key_height),
            );

            // 绘制按键背景
            let bg_color = Color32::from_gray(60);
            let border_color = Color32::from_gray(100);
            let text_color = Color32::from_gray(220);

            // 绘制圆角矩形背景
            painter.rect(
                key_rect,
                CornerRadius::same(3),
                bg_color,
                Stroke::new(1.0, border_color),
                egui::StrokeKind::Middle,
            );

            // 绘制内阴影效果（底部和右侧）
            let shadow_color = Color32::from_black_alpha(80);
            painter.rect(
                key_rect.shrink(1.0),
                CornerRadius::same(2),
                Color32::TRANSPARENT,
                Stroke::new(1.0, shadow_color),
                egui::StrokeKind::Middle,
            );

            // 绘制按键文字
            painter.text(
                key_rect.center(),
                Align2::CENTER_CENTER,
                key,
                FontId::monospace(11.0),
                text_color,
            );

            cursor_x += key_width;

            // 绘制连接符+
            if i < shortcut.len() - 1 {
                let plus_x = cursor_x + spacing + plus_width / 2.0;
                painter.text(
                    egui::pos2(plus_x, center_y),
                    Align2::CENTER_CENTER,
                    "+",
                    FontId::monospace(10.0),
                    Color32::from_gray(150),
                );
                cursor_x += plus_width + spacing * 2.0;
            }
        }
    });
}
