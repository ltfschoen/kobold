use kobold::prelude::*;
use wasm_bindgen::prelude::*;

use fast_qr::qr::QRBuilder;
use web_sys::CanvasRenderingContext2d;

#[component]
pub fn KoboldQR(data: &str) -> impl Html {
    let qr = QRBuilder::new(data).build().ok()?;
    let size = qr.size * 8;

    Some(
        html! {
            <canvas width={size} height={size} style="width: 200px; height: 200px;" />
        }
        .on_render(move |canvas| {
            let ctx = match canvas.get_context("2d") {
                Ok(Some(ctx)) => ctx.unchecked_into::<CanvasRenderingContext2d>(),
                _ => return,
            };

            ctx.clear_rect(0., 0., size as f64, size as f64);

            for (y, row) in qr.data.chunks(qr.size).take(qr.size).enumerate() {
                let mut row = row.iter().enumerate();

                while let Some((x, m)) = row.next() {
                    if !m.value() {
                        continue;
                    }

                    let w = 1 + (&mut row).take_while(|(_, m)| m.value()).count();

                    ctx.fill_rect((8 * x) as f64, (8 * y) as f64, (w * 8) as f64, 8.)
                }
            }
        }),
    )
}