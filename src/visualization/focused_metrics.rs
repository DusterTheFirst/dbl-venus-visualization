use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use eframe::{
    egui::{
        plot::{uniform_grid_spacer, Corner, Legend, Line, Plot, Points, Value, Values},
        Ui,
    },
    epaint::Color32,
};

use crate::serial::metric::{name::MetricName, timestamp::Timestamp, value::MetricValue};

fn label_formatter(name: &str, value: &Value) -> String {
    format!("{name}\n{}\n@ {}", value.y, x_value_formatter(value.x))
}

fn x_value_formatter(x: f64) -> String {
    format!(
        "{}{}",
        if x.is_sign_negative() { "-" } else { "" },
        Timestamp::from_millis(x.abs() as u32)
    )
}

fn color_from_metric_name(metric_name: &MetricName) -> Color32 {
    let mut hasher = DefaultHasher::new();

    metric_name.hash(&mut hasher);

    // Get random but deterministic color per line
    let index = hasher.finish();

    let color = colorous::RAINBOW.eval_rational(index as usize, u64::MAX as usize);

    Color32::from_rgb(color.r, color.g, color.b)
}

pub fn focused_metrics_plot<'ui, 'iter>(
    ui: &'ui mut Ui,
    focused_metrics: impl Iterator<
            Item = (
                &'iter MetricName,
                impl Iterator<Item = &'iter (Timestamp, MetricValue)>,
            ),
        > + 'iter,
    connect_the_dots: bool,
) {
    Plot::new("focused_metrics")
        .include_y(0.0)
        .include_y(1.0)
        .x_axis_formatter(|x, _range| x_value_formatter(x))
        .x_grid_spacer(uniform_grid_spacer(|_| [60.0 * 1000.0, 1000.0, 100.0]))
        .label_formatter(label_formatter)
        .legend(Legend::default().position(Corner::LeftTop))
        .show(ui, |ui| {
            for (metric_name, metric_values) in focused_metrics {
                let values = metric_values
                    .map(|(timestamp, value)| {
                        Value::new(
                            timestamp.timestamp(),
                            value
                                .as_float()
                                .or_else(|| value.as_unsigned_integer().map(|int| int as f64))
                                .or_else(|| value.as_signed_integer().map(|int| int as f64))
                                .or_else(|| {
                                    value.as_bool().map(|bool| if bool { 1.0 } else { 0.0 })
                                })
                                .unwrap_or(f64::NAN),
                        )
                    })
                    .collect::<Vec<_>>();

                let color = color_from_metric_name(metric_name);

                if connect_the_dots {
                    ui.line(
                        Line::new(Values::from_values(values.clone()))
                            .name(metric_name.to_string())
                            .color(color),
                    );
                }
                ui.points(
                    Points::new(Values::from_values(values))
                        .radius(2.0)
                        .name(metric_name.to_string())
                        .color(color),
                );
            }
        });
}
