//! Cairo backend implementing the CanvasRenderingContext2D-like traits behind
//! the optional `cairo` crate feature. The implementation favors fidelity where
//! practical and uses no-ops or TODOs for APIs that Cairo does not support
//! directly (shadows, image smoothing toggles, patterns, image data upload).

use cairo::{
    Context, Extend, FillRule as CairoFillRule, Format, ImageSurface, LineCap as CairoLineCap, LineJoin as CairoLineJoin,
    Operator, SurfacePattern, Filter,
};

use crate::api::*;
use crate::error::{Result, LignumError};

/// Adapter that translates CanvasRenderingContext2D calls into Cairo operations.
pub struct CairoCanvas {
    ctx: Context,
    fill_style: Paint,
    stroke_style: Paint,
    global_alpha: f64,
    composite: CompositeOperation,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    shadow_color: String,
    image_smoothing_enabled: bool,
    image_smoothing_quality: ImageSmoothingQuality,
    line_dash_offset: f64,
    font: String,
    text_align: TextAlign,
    text_baseline: TextBaseline,
    direction: Direction,
}

impl CairoCanvas {
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx,
            fill_style: Paint::Color("#000000".into()),
            stroke_style: Paint::Color("#000000".into()),
            global_alpha: 1.0,
            composite: CompositeOperation::SourceOver,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: "rgba(0,0,0,0)".into(),
            image_smoothing_enabled: true,
            image_smoothing_quality: ImageSmoothingQuality::Medium,
            line_dash_offset: 0.0,
            font: "16px Sans".into(),
            text_align: TextAlign::Start,
            text_baseline: TextBaseline::Alphabetic,
            direction: Direction::Inherit,
        }
    }

    fn apply_composite(&self) {
        self.ctx
            .set_operator(map_composite(self.composite.clone()));
    }

    fn apply_paint(&self, paint: &Paint) -> Result<()> {
        match paint {
            Paint::Color(s) => {
                let (r, g, b, a) = parse_color(s);
                let a = a * self.global_alpha;
                self.ctx.set_source_rgba(r, g, b, a);
            }
            Paint::Gradient(grad) => match &grad.kind {
                GradientKind::Linear { x0, y0, x1, y1 } => {
                    let pattern = cairo::LinearGradient::new(*x0, *y0, *x1, *y1);
                    for stop in &grad.stops {
                        let (r, g, b, a) = parse_color(&stop.color);
                        pattern.add_color_stop_rgba(stop.offset, r, g, b, a * self.global_alpha);
                    }
                    self.ctx.set_source(&pattern)?;
                }
                GradientKind::Radial {
                    x0,
                    y0,
                    r0,
                    x1,
                    y1,
                    r1,
                } => {
                    let pattern = cairo::RadialGradient::new(*x0, *y0, *r0, *x1, *y1, *r1);
                    for stop in &grad.stops {
                        let (r, g, b, a) = parse_color(&stop.color);
                        pattern.add_color_stop_rgba(stop.offset, r, g, b, a * self.global_alpha);
                    }
                    self.ctx.set_source(&pattern)?;
                }
            },
            Paint::Pattern(_p) => {
                // Proper pattern support requires access to concrete image sources.
                todo!("Pattern painting is not implemented for Cairo backend yet");
            }
        }

        Ok(())
    }

    fn apply_font(&self) {
        let (size, family) = parse_font(&self.font);
        self.ctx
            .select_font_face(family, cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        self.ctx.set_font_size(size);
    }
}

impl CanvasState for CairoCanvas {
    fn save(&mut self) -> Result<()> {
        self.ctx.save()?;
        Ok(())
    }

    fn restore(&mut self) -> Result<()> {
        self.ctx.restore()?;
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        self.ctx.identity_matrix();
        self.ctx.reset_clip();
        self.ctx.set_dash(&[], 0.0);
        self.line_dash_offset = 0.0;
        Ok(())
    }

    fn set_global_alpha(&mut self, value: f64) -> Result<()> {
        self.global_alpha = value;
        Ok(())
    }

    fn global_alpha(&self) -> Result<f64> {
        Ok(self.global_alpha)
    }

    fn set_global_composite_operation(&mut self, op: CompositeOperation) -> Result<()> {
        self.composite = op;
        self.apply_composite();
        Ok(())
    }

    fn global_composite_operation(&self) -> Result<CompositeOperation> {
        Ok(self.composite.clone())
    }

    fn set_image_smoothing_enabled(&mut self, enabled: bool) -> Result<()> {
        self.image_smoothing_enabled = enabled;
        Ok(())
    }

    fn image_smoothing_enabled(&self) -> Result<bool> {
        Ok(self.image_smoothing_enabled)
    }

    fn set_image_smoothing_quality(&mut self, quality: ImageSmoothingQuality) -> Result<()> {
        self.image_smoothing_quality = quality;
        Ok(())
    }

    fn image_smoothing_quality(&self) -> Result<ImageSmoothingQuality> {
        Ok(self.image_smoothing_quality.clone())
    }
}

impl CanvasTransforms for CairoCanvas {
    fn scale(&mut self, x: f64, y: f64) -> Result<()> {
        self.ctx.scale(x, y);
        Ok(())
    }

    fn rotate(&mut self, radians: f64) -> Result<()> {
        self.ctx.rotate(radians);
        Ok(())
    }

    fn translate(&mut self, x: f64, y: f64) -> Result<()> {
        self.ctx.translate(x, y);
        Ok(())
    }

    fn transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()> {
        let matrix = cairo::Matrix::new(a, b, c, d, e, f);
        self.ctx.transform(matrix);
        Ok(())
    }

    fn set_transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()> {
        let matrix = cairo::Matrix::new(a, b, c, d, e, f);
        self.ctx.set_matrix(matrix);
        Ok(())
    }

    fn reset_transform(&mut self) -> Result<()> {
        self.ctx.identity_matrix();
        Ok(())
    }
}

impl CairoCanvas {
    fn image_surface_from_rgba(&self, image: &dyn CanvasImageSource) -> Result<ImageSurface> {
        let width = image.width();
        let height = image.height();
        let data = image
            .data_rgba()
            .ok_or_else(|| LignumError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "CanvasImageSource missing RGBA data",
            ))))?;

        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|v| v.checked_mul(4))
            .ok_or_else(|| LignumError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "image dimensions overflow",
            ))))?;

        if data.len() != expected {
            return Err(LignumError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "RGBA buffer length does not match width*height*4",
            ))));
        }

        let mut buf = vec![0u8; expected];
        for (i, chunk) in data.chunks_exact(4).enumerate() {
            let r = chunk[0] as u16;
            let g = chunk[1] as u16;
            let b = chunk[2] as u16;
            let a = chunk[3] as u16;
            let pr = (r * a + 127) / 255;
            let pg = (g * a + 127) / 255;
            let pb = (b * a + 127) / 255;
            let idx = i * 4;
            // Cairo ARgb32 expects premultiplied alpha with native-endian (BGRA on little-endian).
            buf[idx] = pb as u8;
            buf[idx + 1] = pg as u8;
            buf[idx + 2] = pr as u8;
            buf[idx + 3] = a as u8;
        }

        let stride = (width * 4) as i32;
        let surface = ImageSurface::create_for_data(buf, Format::ARgb32, width as i32, height as i32, stride)?;
        Ok(surface)
    }

    fn make_image_pattern(&self, surface: &ImageSurface) -> SurfacePattern {
        let pattern = SurfacePattern::create(surface);
        let filter = if !self.image_smoothing_enabled {
            Filter::Nearest
        } else {
            match self.image_smoothing_quality {
                ImageSmoothingQuality::Low => Filter::Fast,
                ImageSmoothingQuality::Medium => Filter::Good,
                ImageSmoothingQuality::High => Filter::Best,
            }
        };
        pattern.set_filter(filter);
        pattern.set_extend(Extend::None);
        pattern
    }
}

impl CanvasCompositing for CairoCanvas {
    fn set_shadow_offset_x(&mut self, value: f64) -> Result<()> {
        self.shadow_offset_x = value;
        Ok(())
    }

    fn shadow_offset_x(&self) -> Result<f64> {
        Ok(self.shadow_offset_x)
    }

    fn set_shadow_offset_y(&mut self, value: f64) -> Result<()> {
        self.shadow_offset_y = value;
        Ok(())
    }

    fn shadow_offset_y(&self) -> Result<f64> {
        Ok(self.shadow_offset_y)
    }

    fn set_shadow_blur(&mut self, value: f64) -> Result<()> {
        self.shadow_blur = value;
        Ok(())
    }

    fn shadow_blur(&self) -> Result<f64> {
        Ok(self.shadow_blur)
    }

    fn set_shadow_color(&mut self, value: String) -> Result<()> {
        self.shadow_color = value;
        Ok(())
    }

    fn shadow_color(&self) -> Result<String> {
        Ok(self.shadow_color.clone())
    }
}

impl CanvasLineStyles for CairoCanvas {
    fn set_line_width(&mut self, value: f64) -> Result<()> {
        self.ctx.set_line_width(value);
        Ok(())
    }

    fn line_width(&self) -> Result<f64> {
        Ok(self.ctx.line_width())
    }

    fn set_line_cap(&mut self, value: LineCap) -> Result<()> {
        self.ctx.set_line_cap(map_line_cap(value));
        Ok(())
    }

    fn line_cap(&self) -> Result<LineCap> {
        Ok(map_line_cap_back(self.ctx.line_cap()))
    }

    fn set_line_join(&mut self, value: LineJoin) -> Result<()> {
        self.ctx.set_line_join(map_line_join(value));
        Ok(())
    }

    fn line_join(&self) -> Result<LineJoin> {
        Ok(map_line_join_back(self.ctx.line_join()))
    }

    fn set_miter_limit(&mut self, value: f64) -> Result<()> {
        self.ctx.set_miter_limit(value);
        Ok(())
    }

    fn miter_limit(&self) -> Result<f64> {
        Ok(self.ctx.miter_limit())
    }

    fn set_line_dash(&mut self, segments: Vec<f64>) -> Result<()> {
        self.ctx.set_dash(&segments, self.line_dash_offset);
        Ok(())
    }

    fn line_dash(&self) -> Result<Vec<f64>> {
        Ok(self.ctx.dash().0)
    }

    fn set_line_dash_offset(&mut self, value: f64) -> Result<()> {
        self.line_dash_offset = value;
        let (segments, _) = self.ctx.dash();
        self.ctx.set_dash(&segments, self.line_dash_offset);
        Ok(())
    }

    fn line_dash_offset(&self) -> Result<f64> {
        Ok(self.ctx.dash().1)
    }
}

impl CanvasFillStrokeStyles for CairoCanvas {
    fn set_fill_style(&mut self, style: Paint) -> Result<()> {
        self.fill_style = style;
        Ok(())
    }

    fn fill_style(&self) -> Result<Paint> {
        Ok(self.fill_style.clone())
    }

    fn set_stroke_style(&mut self, style: Paint) -> Result<()> {
        self.stroke_style = style;
        Ok(())
    }

    fn stroke_style(&self) -> Result<Paint> {
        Ok(self.stroke_style.clone())
    }

    fn create_linear_gradient(
        &mut self,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    ) -> Result<CanvasGradient> {
        Ok(CanvasGradient {
            kind: GradientKind::Linear { x0, y0, x1, y1 },
            stops: Vec::new(),
        })
    }

    fn create_radial_gradient(
        &mut self,
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
    ) -> Result<CanvasGradient> {
        Ok(CanvasGradient {
            kind: GradientKind::Radial {
                x0,
                y0,
                r0,
                x1,
                y1,
                r1,
            },
            stops: Vec::new(),
        })
    }

    fn create_pattern(
        &mut self,
        _image: &dyn CanvasImageSource,
        repetition: PatternRepetition,
    ) -> Result<CanvasPattern> {
        Ok(CanvasPattern {
            repetition,
            transform: None,
        })
    }
}

impl CanvasRectangles for CairoCanvas {
    fn clear_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        self.ctx.save()?;
        self.ctx.rectangle(x, y, w, h);
        self.ctx.set_operator(Operator::Clear);
        self.ctx.fill()?;
        self.ctx.restore()?;
        self.apply_composite();
        Ok(())
    }

    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        self.ctx.rectangle(x, y, w, h);
        self.apply_paint(&self.fill_style)?;
        self.ctx.fill()?;
        Ok(())
    }

    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        self.ctx.rectangle(x, y, w, h);
        self.apply_paint(&self.stroke_style)?;
        self.ctx.stroke()?;
        Ok(())
    }
}

impl CanvasPaths for CairoCanvas {
    fn begin_path(&mut self) -> Result<()> {
        self.ctx.new_path();
        Ok(())
    }

    fn close_path(&mut self) -> Result<()> {
        self.ctx.close_path();
        Ok(())
    }

    fn move_to(&mut self, x: f64, y: f64) -> Result<()> {
        self.ctx.move_to(x, y);
        Ok(())
    }

    fn line_to(&mut self, x: f64, y: f64) -> Result<()> {
        self.ctx.line_to(x, y);
        Ok(())
    }

    fn bezier_curve_to(
        &mut self,
        cp1x: f64,
        cp1y: f64,
        cp2x: f64,
        cp2y: f64,
        x: f64,
        y: f64,
    ) -> Result<()> {
        self.ctx.curve_to(cp1x, cp1y, cp2x, cp2y, x, y);
        Ok(())
    }

    fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) -> Result<()> {
        let (sx, sy) = self.ctx.current_point()?;
        self.ctx.curve_to(
            sx + 2.0 / 3.0 * (cpx - sx),
            sy + 2.0 / 3.0 * (cpy - sy),
            x + 2.0 / 3.0 * (cpx - x),
            y + 2.0 / 3.0 * (cpy - y),
            x,
            y,
        );
        Ok(())
    }

    fn arc(
        &mut self,
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    ) -> Result<()> {
        if ccw {
            self.ctx.arc_negative(x, y, radius, start_angle, end_angle);
        } else {
            self.ctx.arc(x, y, radius, start_angle, end_angle);
        }
        Ok(())
    }

    fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) -> Result<()> {
        let (x0, y0) = self.ctx.current_point()?;
        let r = radius;

        // Degenerate cases: treat as straight segments.
        if r == 0.0
            || ((x0 - x1).abs() < 1e-9 && (y0 - y1).abs() < 1e-9)
            || ((x1 - x2).abs() < 1e-9 && (y1 - y2).abs() < 1e-9)
        {
            self.line_to(x1, y1)?;
            return Ok(());
        }

        let v1 = (x0 - x1, y0 - y1);
        let v2 = (x2 - x1, y2 - y1);
        let len1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
        let len2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();
        if len1 < 1e-9 || len2 < 1e-9 {
            self.line_to(x1, y1)?;
            return Ok(());
        }

        let v1n = (v1.0 / len1, v1.1 / len1);
        let v2n = (v2.0 / len2, v2.1 / len2);
        let dot = (v1n.0 * v2n.0 + v1n.1 * v2n.1).clamp(-1.0, 1.0);

        // Collinear: draw straight.
        if (1.0 - dot).abs() < 1e-6 || (1.0 + dot).abs() < 1e-6 {
            self.line_to(x1, y1)?;
            return Ok(());
        }

        let angle = dot.acos();
        let tan_half = (angle / 2.0).tan();
        if tan_half.abs() < 1e-9 {
            self.line_to(x1, y1)?;
            return Ok(());
        }
        let dist = r / tan_half;

        let tp1 = (x1 + v1n.0 * dist, y1 + v1n.1 * dist);
        let tp2 = (x1 + v2n.0 * dist, y1 + v2n.1 * dist);

        let cross = v1n.0 * v2n.1 - v1n.1 * v2n.0;
        let mut n1 = (-v1n.1, v1n.0);
        if cross < 0.0 {
            n1 = (v1n.1, -v1n.0);
        }
        let center = (tp1.0 + n1.0 * r, tp1.1 + n1.1 * r);
        let start_ang = (tp1.1 - center.1).atan2(tp1.0 - center.0);
        let end_ang = (tp2.1 - center.1).atan2(tp2.0 - center.0);

        self.line_to(tp1.0, tp1.1)?;
        if cross > 0.0 {
            self.ctx.arc(center.0, center.1, r, start_ang, end_ang);
        } else {
            self.ctx.arc_negative(center.0, center.1, r, start_ang, end_ang);
        }
        Ok(())
    }

    fn ellipse(
        &mut self,
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    ) -> Result<()> {
        self.ctx.save()?;
        self.ctx.translate(x, y);
        self.ctx.rotate(rotation);
        self.ctx.scale(radius_x, radius_y);
        self.arc(0.0, 0.0, 1.0, start_angle, end_angle, ccw)?;
        self.ctx.restore()?;
        Ok(())
    }

    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        self.ctx.rectangle(x, y, w, h);
        Ok(())
    }

    fn round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, radii: &[f64]) -> Result<()> {
        let r = radii.first().cloned().unwrap_or(0.0);
        let r = r.min(w / 2.0).min(h / 2.0);
        let right = x + w;
        let bottom = y + h;

        self.begin_path()?;
        self.ctx.arc(
            x + r,
            y + r,
            r,
            std::f64::consts::PI,
            1.5 * std::f64::consts::PI,
        );
        self.ctx
            .arc(right - r, y + r, r, 1.5 * std::f64::consts::PI, 0.0);
        self.ctx
            .arc(right - r, bottom - r, r, 0.0, 0.5 * std::f64::consts::PI);
        self.ctx.arc(
            x + r,
            bottom - r,
            r,
            0.5 * std::f64::consts::PI,
            std::f64::consts::PI,
        );
        self.close_path()?;
        Ok(())
    }

    fn fill(&mut self, fill_rule: FillRule) -> Result<()> {
        self.ctx.set_fill_rule(map_fill_rule(fill_rule));
        self.apply_paint(&self.fill_style)?;
        self.ctx.fill()?;
        Ok(())
    }

    fn stroke(&mut self) -> Result<()> {
        self.apply_paint(&self.stroke_style)?;
        self.ctx.stroke()?;
        Ok(())
    }

    fn clip(&mut self, fill_rule: FillRule) -> Result<()> {
        self.ctx.set_fill_rule(map_fill_rule(fill_rule));
        self.ctx.clip();
        Ok(())
    }

    fn is_point_in_path(&self, x: f64, y: f64, _opts: HitOptions) -> Result<bool> {
        Ok(self.ctx.in_fill(x, y)?)
    }

    fn is_point_in_stroke(&self, x: f64, y: f64) -> Result<bool> {
        Ok(self.ctx.in_stroke(x, y)?)
    }
}

impl CanvasText for CairoCanvas {
    fn set_font(&mut self, value: String) -> Result<()> {
        self.font = value;
        Ok(())
    }

    fn font(&self) -> Result<String> {
        Ok(self.font.clone())
    }

    fn set_text_align(&mut self, value: TextAlign) -> Result<()> {
        self.text_align = value;
        Ok(())
    }

    fn text_align(&self) -> Result<TextAlign> {
        Ok(self.text_align.clone())
    }

    fn set_text_baseline(&mut self, value: TextBaseline) -> Result<()> {
        self.text_baseline = value;
        Ok(())
    }

    fn text_baseline(&self) -> Result<TextBaseline> {
        Ok(self.text_baseline.clone())
    }

    fn set_direction(&mut self, value: Direction) -> Result<()> {
        self.direction = value;
        Ok(())
    }

    fn direction(&self) -> Result<Direction> {
        Ok(self.direction.clone())
    }

    fn fill_text(&mut self, text: &str, x: f64, y: f64, _max_width: Option<f64>) -> Result<()> {
        self.apply_font();
        self.apply_paint(&self.fill_style)?;
        let (tx, ty) = adjust_text_position(
            &self.ctx,
            text,
            x,
            y,
            self.text_align.clone(),
            self.text_baseline.clone(),
        )?;
        self.ctx.move_to(tx, ty);
        self.ctx.show_text(text)?;
        Ok(())
    }

    fn stroke_text(&mut self, text: &str, x: f64, y: f64, _max_width: Option<f64>) -> Result<()> {
        self.apply_font();
        self.apply_paint(&self.stroke_style)?;
        let (tx, ty) = adjust_text_position(
            &self.ctx,
            text,
            x,
            y,
            self.text_align.clone(),
            self.text_baseline.clone(),
        )?;
        self.ctx.move_to(tx, ty);
        self.ctx.text_path(text);
        self.ctx.stroke()?;
        Ok(())
    }

    fn measure_text(&self, text: &str) -> Result<TextMetrics> {
        self.apply_font();
        let extents = self.ctx.text_extents(text)?;
        Ok(TextMetrics {
            width: extents.width(),
        })
    }
}

impl CanvasImageData for CairoCanvas {
    fn create_image_data(&mut self, width: u32, height: u32) -> Result<ImageData> {
        Ok(ImageData {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
        })
    }

    fn get_image_data(&self, _sx: u32, _sy: u32, sw: u32, sh: u32) -> Result<ImageData> {
        // Reading back from Cairo surfaces would require access to the surface.
        // Provide a zeroed buffer placeholder for now.
        Ok(ImageData {
            width: sw,
            height: sh,
            data: vec![0; (sw * sh * 4) as usize],
        })
    }

    fn put_image_data(&mut self, _data: &ImageData, _dx: f64, _dy: f64) -> Result<()> {
        // Not implemented: would need to write pixels into a surface.
        todo!("put_image_data is not implemented for Cairo backend yet");
    }

    fn put_image_data_dirty(
        &mut self,
        data: &ImageData,
        dx: f64,
        dy: f64,
        _dirty_x: u32,
        _dirty_y: u32,
        _dirty_width: u32,
        _dirty_height: u32,
    ) -> Result<()> {
        self.put_image_data(data, dx, dy)
    }
}

impl CanvasDrawImage for CairoCanvas {
    fn draw_image(&mut self, image: &dyn CanvasImageSource, dx: f64, dy: f64) -> Result<()> {
        let surface = self.image_surface_from_rgba(image)?;
        let pattern = self.make_image_pattern(&surface);

        self.ctx.save()?;
        self.apply_composite();
        self.ctx.set_source(&pattern)?;
        self.ctx.rectangle(dx, dy, image.width() as f64, image.height() as f64);
        self.ctx.clip();
        self.ctx.paint_with_alpha(self.global_alpha)?;
        self.ctx.restore()?;
        Ok(())
    }

    fn draw_image_scaled(
        &mut self,
        image: &dyn CanvasImageSource,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> Result<()> {
        let surface = self.image_surface_from_rgba(image)?;
        let pattern = self.make_image_pattern(&surface);
        let scale_x = dw / image.width() as f64;
        let scale_y = dh / image.height() as f64;

        self.ctx.save()?;
        self.apply_composite();
        self.ctx.translate(dx, dy);
        self.ctx.scale(scale_x, scale_y);
        self.ctx.set_source(&pattern)?;
        self.ctx.rectangle(0.0, 0.0, image.width() as f64, image.height() as f64);
        self.ctx.clip();
        self.ctx.paint_with_alpha(self.global_alpha)?;
        self.ctx.restore()?;
        Ok(())
    }

    fn draw_image_subrect(
        &mut self,
        image: &dyn CanvasImageSource,
        sx: f64,
        sy: f64,
        sw: f64,
        sh: f64,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> Result<()> {
        let surface = self.image_surface_from_rgba(image)?;
        let pattern = self.make_image_pattern(&surface);
        let scale_x = dw / sw;
        let scale_y = dh / sh;

        self.ctx.save()?;
        self.apply_composite();
        self.ctx.rectangle(dx, dy, dw, dh);
        self.ctx.clip();
        self.ctx.translate(dx, dy);
        self.ctx.scale(scale_x, scale_y);
        self.ctx.translate(-sx, -sy);
        self.ctx.set_source(&pattern)?;
        self.ctx.paint_with_alpha(self.global_alpha)?;
        self.ctx.restore()?;
        Ok(())
    }
}

fn map_line_cap(cap: LineCap) -> CairoLineCap {
    match cap {
        LineCap::Butt => CairoLineCap::Butt,
        LineCap::Round => CairoLineCap::Round,
        LineCap::Square => CairoLineCap::Square,
    }
}

fn map_line_cap_back(cap: CairoLineCap) -> LineCap {
    match cap {
        CairoLineCap::Butt => LineCap::Butt,
        CairoLineCap::Round => LineCap::Round,
        CairoLineCap::Square => LineCap::Square,
        _ => unreachable!("non-exhaustive cairo LineCap")
    }
}

fn map_line_join(join: LineJoin) -> CairoLineJoin {
    match join {
        LineJoin::Bevel => CairoLineJoin::Bevel,
        LineJoin::Miter => CairoLineJoin::Miter,
        LineJoin::Round => CairoLineJoin::Round,
    }
}

fn map_line_join_back(join: CairoLineJoin) -> LineJoin {
    match join {
        CairoLineJoin::Bevel => LineJoin::Bevel,
        CairoLineJoin::Miter => LineJoin::Miter,
        CairoLineJoin::Round => LineJoin::Round,
        _ => unreachable!("non-exhaustive cairo LineJoin")
    }
}

fn map_fill_rule(rule: FillRule) -> CairoFillRule {
    match rule {
        FillRule::NonZero => CairoFillRule::Winding,
        FillRule::EvenOdd => CairoFillRule::EvenOdd,
    }
}

fn map_composite(op: CompositeOperation) -> Operator {
    match op {
        CompositeOperation::SourceOver => Operator::Over,
        CompositeOperation::SourceIn => Operator::In,
        CompositeOperation::SourceOut => Operator::Out,
        CompositeOperation::SourceAtop => Operator::Atop,
        CompositeOperation::DestinationOver => Operator::DestOver,
        CompositeOperation::DestinationIn => Operator::DestIn,
        CompositeOperation::DestinationOut => Operator::DestOut,
        CompositeOperation::DestinationAtop => Operator::DestAtop,
        CompositeOperation::Lighter => Operator::Add,
        CompositeOperation::Copy => Operator::Source,
        CompositeOperation::Xor => Operator::Xor,
        CompositeOperation::Multiply => Operator::Multiply,
        CompositeOperation::Screen => Operator::Screen,
        CompositeOperation::Overlay => Operator::Overlay,
        CompositeOperation::Darken => Operator::Darken,
        CompositeOperation::Lighten => Operator::Lighten,
        CompositeOperation::ColorDodge => Operator::ColorDodge,
        CompositeOperation::ColorBurn => Operator::ColorBurn,
        CompositeOperation::HardLight => Operator::HardLight,
        CompositeOperation::SoftLight => Operator::SoftLight,
        CompositeOperation::Difference => Operator::Difference,
        CompositeOperation::Exclusion => Operator::Exclusion,
        CompositeOperation::Hue => Operator::HslHue,
        CompositeOperation::Saturation => Operator::HslSaturation,
        CompositeOperation::Color => Operator::HslColor,
        CompositeOperation::Luminosity => Operator::HslLuminosity,
    }
}

fn parse_color(color: &str) -> (f64, f64, f64, f64) {
    let c = color.trim();
    if let Some(hex) = c.strip_prefix('#') {
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                return (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0, 1.0);
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
                return (
                    r as f64 / 255.0,
                    g as f64 / 255.0,
                    b as f64 / 255.0,
                    a as f64 / 255.0,
                );
            }
            _ => {}
        }
    }

    // Fallback to opaque black if parsing fails.
    (0.0, 0.0, 0.0, 1.0)
}

fn parse_font(font: &str) -> (f64, &str) {
    // Minimal parser for strings like "16px Sans".
    let mut size = 16.0;
    let mut family = "Sans";
    for part in font.split_whitespace() {
        if let Some(px) = part.strip_suffix("px") {
            if let Ok(v) = px.parse::<f64>() {
                size = v;
            }
        } else {
            family = part;
        }
    }
    (size, family)
}

fn adjust_text_position(
    ctx: &Context,
    text: &str,
    x: f64,
    y: f64,
    align: TextAlign,
    baseline: TextBaseline,
) -> Result<(f64, f64)> {
    let extents = ctx.text_extents(text)?;
    let mut tx = x;
    let mut ty = y;

    tx -= match align {
        TextAlign::Left | TextAlign::Start => 0.0,
        TextAlign::Center => extents.width() / 2.0,
        TextAlign::Right | TextAlign::End => extents.width(),
    };

    ty += match baseline {
        TextBaseline::Top => extents.height(),
        TextBaseline::Hanging => extents.height() * 0.8,
        TextBaseline::Middle => extents.height() * 0.5,
        TextBaseline::Alphabetic => 0.0,
        TextBaseline::Ideographic => extents.height() * 0.1,
        TextBaseline::Bottom => -extents.y_bearing(),
    };

    Ok((tx, ty))
}

impl CanvasRenderingContext2D for CairoCanvas {}
