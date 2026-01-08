//! SVG backend using a streaming XML writer.
//! This is a skeleton; implement Canvas traits to emit SVG elements.

use std::io::Write;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use png::{ColorType, Encoder as PngEncoder};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

use crate::api::{
    CanvasDrawImage, CanvasFillStrokeStyles, CanvasGradient, CanvasImageData, CanvasImageSource,
    CanvasLineStyles, CanvasPaths, CanvasRectangles, CanvasRenderingContext2D, CanvasState,
    CanvasText, CanvasTransforms, CompositeOperation, Direction, FillRule, GradientKind,
    HitOptions, ImageData, ImageSmoothingQuality, LineCap, LineJoin, Paint, PatternRepetition,
    TextAlign, TextBaseline, TextMetrics,
};
use crate::error::{LignumError, Result};

/// Minimal SVG canvas wrapper around `quick_xml::Writer`.
pub struct SvgCanvas<W: Write> {
    writer: Writer<W>,
    open_root: bool,
    #[allow(dead_code)]
    width: f64,
    #[allow(dead_code)]
    height: f64,
    current_path: String,
    current_point: Option<(f64, f64)>,
    subpath_start: Option<(f64, f64)>,
    state: SvgState,
    stack: Vec<SvgState>,
    gradient_counter: usize,
    pattern_counter: usize,
}

impl<W: Write> SvgCanvas<W> {
    /// Create a new SVG canvas that writes into the provided sink, emitting the root `<svg>`.
    /// Width/height are expressed in CSS pixels; a matching `viewBox` is set.
    pub fn new(inner: W, width: f64, height: f64) -> Result<Self> {
        let mut writer = Writer::new_with_indent(inner, b' ', 2);
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        let width_attr = width.to_string();
        let height_attr = height.to_string();
        let view_box_attr = format!("0 0 {} {}", width, height);

        let mut start = BytesStart::new("svg");
        start.push_attribute(("xmlns", "http://www.w3.org/2000/svg"));
        start.push_attribute(("version", "1.1"));
        start.push_attribute(("width", width_attr.as_str()));
        start.push_attribute(("height", height_attr.as_str()));
        start.push_attribute(("viewBox", view_box_attr.as_str()));
        writer.write_event(Event::Start(start))?;

        Ok(Self {
            writer,
            open_root: true,
            width,
            height,
            current_path: String::new(),
            current_point: None,
            subpath_start: None,
            state: SvgState::default(),
            stack: Vec::new(),
            gradient_counter: 0,
            pattern_counter: 0,
        })
    }

    /// Finish the document, closing the root element and returning the inner writer.
    pub fn finish(mut self) -> Result<W> {
        if self.open_root {
            self.writer.write_event(Event::End(BytesEnd::new("svg")))?;
            self.open_root = false;
        }
        Ok(self.writer.into_inner())
    }

    fn not_supported(op: &'static str) -> LignumError {
        LignumError::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("SVG backend does not yet implement {op}"),
        )))
    }

    fn write_empty(&mut self, elem: BytesStart<'_>) -> Result<()> {
        self.writer.write_event(Event::Empty(elem))?;
        Ok(())
    }

    fn paint_to_str(&mut self, paint: &Paint) -> Result<String> {
        match paint {
            Paint::Color(c) => Ok(c.clone()),
            Paint::Gradient(g) => self.gradient_paint(g),
            Paint::Pattern(p) => self.pattern_paint(p),
        }
    }

    fn gradient_paint(&mut self, gradient: &CanvasGradient) -> Result<String> {
        let id = format!("grad{}", self.gradient_counter);
        self.gradient_counter += 1;
        self.write_gradient_def(&id, gradient)?;
        Ok(format!("url(#{})", id))
    }

    fn write_gradient_def(&mut self, id: &str, gradient: &CanvasGradient) -> Result<()> {
        self.writer
            .write_event(Event::Start(BytesStart::new("defs")))?;

        match &gradient.kind {
            GradientKind::Linear { x0, y0, x1, y1 } => {
                let mut elem = BytesStart::new("linearGradient");
                elem.push_attribute(("id", id));
                let x1_attr = x0.to_string();
                let y1_attr = y0.to_string();
                let x2_attr = x1.to_string();
                let y2_attr = y1.to_string();
                elem.push_attribute(("x1", x1_attr.as_str()));
                elem.push_attribute(("y1", y1_attr.as_str()));
                elem.push_attribute(("x2", x2_attr.as_str()));
                elem.push_attribute(("y2", y2_attr.as_str()));
                self.writer.write_event(Event::Start(elem))?;
            }
            GradientKind::Radial {
                x0,
                y0,
                r0,
                x1,
                y1,
                r1,
            } => {
                let mut elem = BytesStart::new("radialGradient");
                elem.push_attribute(("id", id));
                let cx_attr = x1.to_string();
                let cy_attr = y1.to_string();
                let r_attr = r1.to_string();
                let fx_attr = x0.to_string();
                let fy_attr = y0.to_string();
                let fr_attr = r0.to_string();
                elem.push_attribute(("cx", cx_attr.as_str()));
                elem.push_attribute(("cy", cy_attr.as_str()));
                elem.push_attribute(("r", r_attr.as_str()));
                elem.push_attribute(("fx", fx_attr.as_str()));
                elem.push_attribute(("fy", fy_attr.as_str()));
                elem.push_attribute(("fr", fr_attr.as_str()));
                self.writer.write_event(Event::Start(elem))?;
            }
        }

        for stop in &gradient.stops {
            let mut stop_elem = BytesStart::new("stop");
            let offset_attr = stop.offset.to_string();
            stop_elem.push_attribute(("offset", offset_attr.as_str()));
            stop_elem.push_attribute(("stop-color", stop.color.as_str()));
            self.writer.write_event(Event::Empty(stop_elem))?;
        }

        let end_tag = match gradient.kind {
            GradientKind::Linear { .. } => "linearGradient",
            GradientKind::Radial { .. } => "radialGradient",
        };
        self.writer
            .write_event(Event::End(BytesEnd::new(end_tag)))?;
        self.writer.write_event(Event::End(BytesEnd::new("defs")))?;
        Ok(())
    }

    fn pattern_paint(&mut self, pattern: &crate::api::CanvasPattern) -> Result<String> {
        let id = format!("pat{}", self.pattern_counter);
        self.pattern_counter += 1;
        self.write_pattern_def(&id, pattern)?;
        Ok(format!("url(#{})", id))
    }

    fn write_pattern_def(&mut self, id: &str, pattern: &crate::api::CanvasPattern) -> Result<()> {
        self.writer
            .write_event(Event::Start(BytesStart::new("defs")))?;

        let mut elem = BytesStart::new("pattern");
        elem.push_attribute(("id", id));
        // Use a 1x1 tile; without image data we cannot scale to source size.
        elem.push_attribute(("width", "1"));
        elem.push_attribute(("height", "1"));
        elem.push_attribute(("patternUnits", "userSpaceOnUse"));
        match pattern.repetition {
            PatternRepetition::Repeat | PatternRepetition::RepeatX | PatternRepetition::RepeatY => {
            }
            PatternRepetition::NoRepeat => {
                // Still emit a pattern; consumers can treat it as single-tile.
            }
        }
        if let Some(m) = pattern.transform {
            let [a, b, c, d, e, f] = m;
            let transform_attr = format!("matrix({} {} {} {} {} {})", a, b, c, d, e, f);
            elem.push_attribute(("patternTransform", transform_attr.as_str()));
        }
        self.writer.write_event(Event::Start(elem))?;

        // Placeholder transparent rect to keep the pattern valid.
        let mut rect = BytesStart::new("rect");
        rect.push_attribute(("width", "1"));
        rect.push_attribute(("height", "1"));
        rect.push_attribute(("fill", "rgba(0,0,0,0)"));
        self.writer.write_event(Event::Empty(rect))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("pattern")))?;
        self.writer.write_event(Event::End(BytesEnd::new("defs")))?;
        Ok(())
    }

    fn encode_image_as_data_uri(&self, image: &dyn CanvasImageSource) -> Result<String> {
        let width = image.width();
        let height = image.height();
        let data = image
            .data_rgba()
            .ok_or_else(|| Self::not_supported("image source lacks RGBA"))?;

        let mut png_bytes = Vec::new();
        let mut encoder = PngEncoder::new(&mut png_bytes, width, height);
        encoder.set_color(ColorType::Rgba);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(data)?;
        writer.finish()?;

        let encoded = BASE64_STANDARD.encode(png_bytes);
        Ok(format!("data:image/png;base64,{}", encoded))
    }

    fn flush_path_fill(&mut self, fill_rule: FillRule) -> Result<()> {
        if self.current_path.is_empty() {
            return Ok(());
        }
        let fill_paint = self.state.fill_style.clone();
        let fill = self.paint_to_str(&fill_paint)?;
        let opacity_attr = self.state.global_alpha.to_string();
        let mut elem = BytesStart::new("path");
        elem.push_attribute(("d", self.current_path.as_str()));
        elem.push_attribute(("fill", fill.as_str()));
        elem.push_attribute(("stroke", "none"));
        elem.push_attribute((
            "fill-rule",
            match fill_rule {
                FillRule::NonZero => "nonzero",
                FillRule::EvenOdd => "evenodd",
            },
        ));
        if self.state.global_alpha < 1.0 {
            elem.push_attribute(("opacity", opacity_attr.as_str()));
        }
        self.apply_transform_attr(&mut elem);
        self.write_empty(elem)
    }

    fn flush_path_stroke(&mut self) -> Result<()> {
        if self.current_path.is_empty() {
            return Ok(());
        }
        let stroke_paint = self.state.stroke_style.clone();
        let stroke = self.paint_to_str(&stroke_paint)?;
        let stroke_width_attr = self.state.line_width.to_string();
        let opacity_attr = self.state.global_alpha.to_string();
        let mut elem = BytesStart::new("path");
        elem.push_attribute(("d", self.current_path.as_str()));
        elem.push_attribute(("fill", "none"));
        elem.push_attribute(("stroke", stroke.as_str()));
        elem.push_attribute(("stroke-width", stroke_width_attr.as_str()));
        elem.push_attribute((
            "stroke-linecap",
            match self.state.line_cap {
                LineCap::Butt => "butt",
                LineCap::Round => "round",
                LineCap::Square => "square",
            },
        ));
        elem.push_attribute((
            "stroke-linejoin",
            match self.state.line_join {
                LineJoin::Round => "round",
                LineJoin::Bevel => "bevel",
                LineJoin::Miter => "miter",
            },
        ));
        if self.state.global_alpha < 1.0 {
            elem.push_attribute(("opacity", opacity_attr.as_str()));
        }
        if !self.state.line_dash.is_empty() {
            let dash = self
                .state
                .line_dash
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            elem.push_attribute(("stroke-dasharray", dash.as_str()));
        }
        if self.state.line_dash_offset != 0.0 {
            let dash_offset_attr = self.state.line_dash_offset.to_string();
            elem.push_attribute(("stroke-dashoffset", dash_offset_attr.as_str()));
        }
        self.apply_transform_attr(&mut elem);
        self.write_empty(elem)
    }

    fn push_path(&mut self, cmd: &str) {
        if !self.current_path.is_empty() {
            self.current_path.push(' ');
        }
        self.current_path.push_str(cmd);
    }

    fn set_current_point(&mut self, x: f64, y: f64) {
        self.current_point = Some((x, y));
    }

    fn ensure_subpath(&mut self) -> Result<()> {
        if self.current_point.is_none() {
            self.move_to(0.0, 0.0)?;
        }
        Ok(())
    }

    fn append_arc_segments(
        &mut self,
        cx: f64,
        cy: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    ) -> Result<()> {
        let tau = std::f64::consts::PI * 2.0;
        let mut delta = end_angle - start_angle;
        if !ccw {
            while delta < 0.0 {
                delta += tau;
            }
        } else {
            while delta > 0.0 {
                delta -= tau;
            }
        }

        if delta.abs() < 1e-12 {
            return Ok(());
        }

        let mut remaining = delta;
        let mut current_angle = start_angle;
        let max_step = std::f64::consts::PI; // keep segments <= 180deg to avoid degenerate arcs

        while remaining.abs() > 1e-12 {
            let step = if remaining.abs() > max_step {
                max_step.copysign(remaining)
            } else {
                remaining
            };

            let next_angle = current_angle + step;
            let end_x = cx + radius * next_angle.cos();
            let end_y = cy + radius * next_angle.sin();
            let large_arc = if step.abs() >= std::f64::consts::PI - 1e-9 {
                1
            } else {
                0
            };
            let sweep_flag = if step >= 0.0 { 1 } else { 0 };

            self.push_path(&format!(
                "A {} {} 0 {} {} {} {}",
                radius, radius, large_arc, sweep_flag, end_x, end_y
            ));
            self.set_current_point(end_x, end_y);

            current_angle = next_angle;
            remaining -= step;
        }

        Ok(())
    }

    fn apply_transform_attr(&self, elem: &mut BytesStart<'_>) {
        let [a, b, c, d, e, f] = self.state.transform;
        if (a, b, c, d, e, f) != (1.0, 0.0, 0.0, 1.0, 0.0, 0.0) {
            let transform_attr = format!("matrix({} {} {} {} {} {})", a, b, c, d, e, f);
            elem.push_attribute(("transform", transform_attr.as_str()));
        }
    }

    fn multiply_transform(&mut self, m: [f64; 6]) {
        let [a, b, c, d, e, f] = self.state.transform;
        let [na, nb, nc, nd, ne, nf] = m;
        self.state.transform = [
            a * na + c * nb,
            b * na + d * nb,
            a * nc + c * nd,
            b * nc + d * nd,
            a * ne + c * nf + e,
            b * ne + d * nf + f,
        ];
    }
}

#[derive(Clone)]
struct SvgState {
    global_alpha: f64,
    global_composite_operation: CompositeOperation,
    image_smoothing_enabled: bool,
    image_smoothing_quality: ImageSmoothingQuality,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    shadow_color: String,
    line_width: f64,
    line_cap: LineCap,
    line_join: LineJoin,
    miter_limit: f64,
    line_dash: Vec<f64>,
    line_dash_offset: f64,
    fill_style: Paint,
    stroke_style: Paint,
    font: String,
    text_align: TextAlign,
    text_baseline: TextBaseline,
    direction: Direction,
    transform: [f64; 6],
}

impl Default for SvgState {
    fn default() -> Self {
        Self {
            global_alpha: 1.0,
            global_composite_operation: CompositeOperation::SourceOver,
            image_smoothing_enabled: true,
            image_smoothing_quality: ImageSmoothingQuality::Low,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: String::from("rgba(0,0,0,0)"),
            line_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 10.0,
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
            fill_style: Paint::Color(String::from("#000")),
            stroke_style: Paint::Color(String::from("#000")),
            font: String::from("10px sans-serif"),
            text_align: TextAlign::Start,
            text_baseline: TextBaseline::Alphabetic,
            direction: Direction::Inherit,
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}

impl<W: Write> CanvasState for SvgCanvas<W> {
    fn save(&mut self) -> Result<()> {
        self.stack.push(self.state.clone());
        Ok(())
    }

    fn restore(&mut self) -> Result<()> {
        if let Some(state) = self.stack.pop() {
            self.state = state;
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        self.state = SvgState::default();
        Ok(())
    }

    fn set_global_alpha(&mut self, value: f64) -> Result<()> {
        self.state.global_alpha = value;
        Ok(())
    }

    fn global_alpha(&self) -> Result<f64> {
        Ok(self.state.global_alpha)
    }

    fn set_global_composite_operation(&mut self, op: CompositeOperation) -> Result<()> {
        self.state.global_composite_operation = op;
        Ok(())
    }

    fn global_composite_operation(&self) -> Result<CompositeOperation> {
        Ok(self.state.global_composite_operation.clone())
    }

    fn set_image_smoothing_enabled(&mut self, enabled: bool) -> Result<()> {
        self.state.image_smoothing_enabled = enabled;
        Ok(())
    }

    fn image_smoothing_enabled(&self) -> Result<bool> {
        Ok(self.state.image_smoothing_enabled)
    }

    fn set_image_smoothing_quality(&mut self, quality: ImageSmoothingQuality) -> Result<()> {
        self.state.image_smoothing_quality = quality;
        Ok(())
    }

    fn image_smoothing_quality(&self) -> Result<ImageSmoothingQuality> {
        Ok(self.state.image_smoothing_quality.clone())
    }
}

impl<W: Write> CanvasTransforms for SvgCanvas<W> {
    fn scale(&mut self, x: f64, y: f64) -> Result<()> {
        self.multiply_transform([x, 0.0, 0.0, y, 0.0, 0.0]);
        Ok(())
    }

    fn rotate(&mut self, radians: f64) -> Result<()> {
        let (s, c) = radians.sin_cos();
        self.multiply_transform([c, s, -s, c, 0.0, 0.0]);
        Ok(())
    }

    fn translate(&mut self, x: f64, y: f64) -> Result<()> {
        self.multiply_transform([1.0, 0.0, 0.0, 1.0, x, y]);
        Ok(())
    }

    fn transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()> {
        self.multiply_transform([a, b, c, d, e, f]);
        Ok(())
    }

    fn set_transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()> {
        self.state.transform = [a, b, c, d, e, f];
        Ok(())
    }

    fn reset_transform(&mut self) -> Result<()> {
        self.state.transform = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        Ok(())
    }
}

impl<W: Write> crate::api::CanvasCompositing for SvgCanvas<W> {
    fn set_shadow_offset_x(&mut self, value: f64) -> Result<()> {
        self.state.shadow_offset_x = value;
        Ok(())
    }

    fn shadow_offset_x(&self) -> Result<f64> {
        Ok(self.state.shadow_offset_x)
    }

    fn set_shadow_offset_y(&mut self, value: f64) -> Result<()> {
        self.state.shadow_offset_y = value;
        Ok(())
    }

    fn shadow_offset_y(&self) -> Result<f64> {
        Ok(self.state.shadow_offset_y)
    }

    fn set_shadow_blur(&mut self, value: f64) -> Result<()> {
        self.state.shadow_blur = value;
        Ok(())
    }

    fn shadow_blur(&self) -> Result<f64> {
        Ok(self.state.shadow_blur)
    }

    fn set_shadow_color(&mut self, value: String) -> Result<()> {
        self.state.shadow_color = value;
        Ok(())
    }

    fn shadow_color(&self) -> Result<String> {
        Ok(self.state.shadow_color.clone())
    }
}

impl<W: Write> CanvasLineStyles for SvgCanvas<W> {
    fn set_line_width(&mut self, value: f64) -> Result<()> {
        self.state.line_width = value;
        Ok(())
    }

    fn line_width(&self) -> Result<f64> {
        Ok(self.state.line_width)
    }

    fn set_line_cap(&mut self, value: LineCap) -> Result<()> {
        self.state.line_cap = value;
        Ok(())
    }

    fn line_cap(&self) -> Result<LineCap> {
        Ok(self.state.line_cap.clone())
    }

    fn set_line_join(&mut self, value: LineJoin) -> Result<()> {
        self.state.line_join = value;
        Ok(())
    }

    fn line_join(&self) -> Result<LineJoin> {
        Ok(self.state.line_join.clone())
    }

    fn set_miter_limit(&mut self, value: f64) -> Result<()> {
        self.state.miter_limit = value;
        Ok(())
    }

    fn miter_limit(&self) -> Result<f64> {
        Ok(self.state.miter_limit)
    }

    fn set_line_dash(&mut self, segments: Vec<f64>) -> Result<()> {
        self.state.line_dash = segments;
        Ok(())
    }

    fn line_dash(&self) -> Result<Vec<f64>> {
        Ok(self.state.line_dash.clone())
    }

    fn set_line_dash_offset(&mut self, value: f64) -> Result<()> {
        self.state.line_dash_offset = value;
        Ok(())
    }

    fn line_dash_offset(&self) -> Result<f64> {
        Ok(self.state.line_dash_offset)
    }
}

impl<W: Write> CanvasFillStrokeStyles for SvgCanvas<W> {
    fn set_fill_style(&mut self, style: Paint) -> Result<()> {
        self.state.fill_style = style;
        Ok(())
    }

    fn fill_style(&self) -> Result<Paint> {
        Ok(self.state.fill_style.clone())
    }

    fn set_stroke_style(&mut self, style: Paint) -> Result<()> {
        self.state.stroke_style = style;
        Ok(())
    }

    fn stroke_style(&self) -> Result<Paint> {
        Ok(self.state.stroke_style.clone())
    }

    fn create_linear_gradient(
        &mut self,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    ) -> Result<crate::api::CanvasGradient> {
        Ok(crate::api::CanvasGradient {
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
    ) -> Result<crate::api::CanvasGradient> {
        Ok(crate::api::CanvasGradient {
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
    ) -> Result<crate::api::CanvasPattern> {
        Ok(crate::api::CanvasPattern {
            repetition,
            transform: None,
        })
    }
}

impl<W: Write> CanvasRectangles for SvgCanvas<W> {
    fn clear_rect(&mut self, _x: f64, _y: f64, _w: f64, _h: f64) -> Result<()> {
        // No-op; clear has no direct SVG equivalent.
        Ok(())
    }

    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        let fill_paint = self.state.fill_style.clone();
        let fill = self.paint_to_str(&fill_paint)?;
        let x_attr = x.to_string();
        let y_attr = y.to_string();
        let w_attr = w.to_string();
        let h_attr = h.to_string();
        let opacity_attr = self.state.global_alpha.to_string();

        let mut elem = BytesStart::new("rect");
        elem.push_attribute(("x", x_attr.as_str()));
        elem.push_attribute(("y", y_attr.as_str()));
        elem.push_attribute(("width", w_attr.as_str()));
        elem.push_attribute(("height", h_attr.as_str()));
        elem.push_attribute(("fill", fill.as_str()));
        if self.state.global_alpha < 1.0 {
            elem.push_attribute(("opacity", opacity_attr.as_str()));
        }
        self.apply_transform_attr(&mut elem);
        self.write_empty(elem)
    }

    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        let stroke_paint = self.state.stroke_style.clone();
        let stroke = self.paint_to_str(&stroke_paint)?;
        let x_attr = x.to_string();
        let y_attr = y.to_string();
        let w_attr = w.to_string();
        let h_attr = h.to_string();
        let stroke_width_attr = self.state.line_width.to_string();

        let mut elem = BytesStart::new("rect");
        elem.push_attribute(("x", x_attr.as_str()));
        elem.push_attribute(("y", y_attr.as_str()));
        elem.push_attribute(("width", w_attr.as_str()));
        elem.push_attribute(("height", h_attr.as_str()));
        elem.push_attribute(("fill", "none"));
        elem.push_attribute(("stroke", stroke.as_str()));
        elem.push_attribute(("stroke-width", stroke_width_attr.as_str()));
        self.apply_transform_attr(&mut elem);
        self.write_empty(elem)
    }
}

impl<W: Write> CanvasPaths for SvgCanvas<W> {
    fn begin_path(&mut self) -> Result<()> {
        self.current_path.clear();
        self.current_point = None;
        self.subpath_start = None;
        Ok(())
    }

    fn close_path(&mut self) -> Result<()> {
        self.push_path("Z");
        if let Some(start) = self.subpath_start {
            self.set_current_point(start.0, start.1);
        }
        Ok(())
    }

    fn move_to(&mut self, x: f64, y: f64) -> Result<()> {
        self.push_path(&format!("M {} {}", x, y));
        self.subpath_start = Some((x, y));
        self.set_current_point(x, y);
        Ok(())
    }

    fn line_to(&mut self, x: f64, y: f64) -> Result<()> {
        if self.current_point.is_none() {
            self.move_to(0.0, 0.0)?;
        }
        self.push_path(&format!("L {} {}", x, y));
        self.set_current_point(x, y);
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
        self.ensure_subpath()?;
        self.push_path(&format!(
            "C {} {}, {} {}, {} {}",
            cp1x, cp1y, cp2x, cp2y, x, y
        ));
        self.set_current_point(x, y);
        Ok(())
    }

    fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) -> Result<()> {
        self.ensure_subpath()?;
        self.push_path(&format!("Q {} {}, {} {}", cpx, cpy, x, y));
        self.set_current_point(x, y);
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
        if radius <= 0.0 {
            return Ok(());
        }

        let start_x = x + radius * start_angle.cos();
        let start_y = y + radius * start_angle.sin();

        match self.current_point {
            Some((px, py)) => {
                if (px - start_x).abs() > 1e-9 || (py - start_y).abs() > 1e-9 {
                    self.line_to(start_x, start_y)?;
                }
            }
            None => {
                self.move_to(start_x, start_y)?;
            }
        }

        self.append_arc_segments(x, y, radius, start_angle, end_angle, ccw)
    }

    fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) -> Result<()> {
        let (x0, y0) = match self.current_point {
            Some(p) => p,
            None => {
                self.move_to(x1, y1)?;
                return Ok(());
            }
        };

        if radius == 0.0
            || ((x0 - x1).abs() < 1e-9 && (y0 - y1).abs() < 1e-9)
            || ((x1 - x2).abs() < 1e-9 && (y1 - y2).abs() < 1e-9)
        {
            return self.line_to(x1, y1);
        }

        let v1 = (x0 - x1, y0 - y1);
        let v2 = (x2 - x1, y2 - y1);
        let len1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
        let len2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();
        if len1 < 1e-9 || len2 < 1e-9 {
            return self.line_to(x1, y1);
        }

        let v1n = (v1.0 / len1, v1.1 / len1);
        let v2n = (v2.0 / len2, v2.1 / len2);
        let dot = (v1n.0 * v2n.0 + v1n.1 * v2n.1).clamp(-1.0, 1.0);

        if (1.0 - dot).abs() < 1e-6 || (1.0 + dot).abs() < 1e-6 {
            return self.line_to(x1, y1);
        }

        let angle = dot.acos();
        let tan_half = (angle / 2.0).tan();
        if tan_half.abs() < 1e-9 {
            return self.line_to(x1, y1);
        }
        let dist = radius / tan_half;

        let tp1 = (x1 + v1n.0 * dist, y1 + v1n.1 * dist);
        let tp2 = (x1 + v2n.0 * dist, y1 + v2n.1 * dist);

        let cross = v1n.0 * v2n.1 - v1n.1 * v2n.0;
        let mut n1 = (-v1n.1, v1n.0);
        if cross < 0.0 {
            n1 = (v1n.1, -v1n.0);
        }
        let center = (tp1.0 + n1.0 * radius, tp1.1 + n1.1 * radius);
        let start_ang = (tp1.1 - center.1).atan2(tp1.0 - center.0);
        let end_ang = (tp2.1 - center.1).atan2(tp2.0 - center.0);

        self.line_to(tp1.0, tp1.1)?;
        self.append_arc_segments(center.0, center.1, radius, start_ang, end_ang, cross < 0.0)
    }

    fn ellipse(
        &mut self,
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        _rotation: f64,
        _start_angle: f64,
        _end_angle: f64,
        _ccw: bool,
    ) -> Result<()> {
        // Approximate as a standalone ellipse element.
        let mut elem = BytesStart::new("ellipse");
        let cx_attr = x.to_string();
        let cy_attr = y.to_string();
        let rx_attr = radius_x.to_string();
        let ry_attr = radius_y.to_string();
        let fill_paint = self.state.fill_style.clone();
        let fill = self.paint_to_str(&fill_paint)?;
        elem.push_attribute(("cx", cx_attr.as_str()));
        elem.push_attribute(("cy", cy_attr.as_str()));
        elem.push_attribute(("rx", rx_attr.as_str()));
        elem.push_attribute(("ry", ry_attr.as_str()));
        elem.push_attribute(("fill", fill.as_str()));
        self.apply_transform_attr(&mut elem);
        self.write_empty(elem)
    }

    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        self.push_path(&format!("M {} {} h {} v {} h {} Z", x, y, w, h, -w));
        self.subpath_start = Some((x, y));
        self.set_current_point(x, y);
        Ok(())
    }

    fn round_rect(&mut self, _x: f64, _y: f64, _w: f64, _h: f64, _radii: &[f64]) -> Result<()> {
        Err(Self::not_supported("round_rect"))
    }

    fn fill(&mut self, fill_rule: FillRule) -> Result<()> {
        self.flush_path_fill(fill_rule)
    }

    fn stroke(&mut self) -> Result<()> {
        self.flush_path_stroke()
    }

    fn clip(&mut self, _fill_rule: FillRule) -> Result<()> {
        Err(Self::not_supported("clip"))
    }

    fn is_point_in_path(&self, _x: f64, _y: f64, _opts: HitOptions) -> Result<bool> {
        Ok(false)
    }

    fn is_point_in_stroke(&self, _x: f64, _y: f64) -> Result<bool> {
        Ok(false)
    }
}

impl<W: Write> CanvasText for SvgCanvas<W> {
    fn set_font(&mut self, value: String) -> Result<()> {
        self.state.font = value;
        Ok(())
    }

    fn font(&self) -> Result<String> {
        Ok(self.state.font.clone())
    }

    fn set_text_align(&mut self, value: TextAlign) -> Result<()> {
        self.state.text_align = value;
        Ok(())
    }

    fn text_align(&self) -> Result<TextAlign> {
        Ok(self.state.text_align.clone())
    }

    fn set_text_baseline(&mut self, value: TextBaseline) -> Result<()> {
        self.state.text_baseline = value;
        Ok(())
    }

    fn text_baseline(&self) -> Result<TextBaseline> {
        Ok(self.state.text_baseline.clone())
    }

    fn set_direction(&mut self, value: Direction) -> Result<()> {
        self.state.direction = value;
        Ok(())
    }

    fn direction(&self) -> Result<Direction> {
        Ok(self.state.direction.clone())
    }

    fn fill_text(&mut self, text: &str, x: f64, y: f64, _max_width: Option<f64>) -> Result<()> {
        let mut elem = BytesStart::new("text");
        let x_attr = x.to_string();
        let y_attr = y.to_string();
        let fill_paint = self.state.fill_style.clone();
        let fill = self.paint_to_str(&fill_paint)?;
        elem.push_attribute(("x", x_attr.as_str()));
        elem.push_attribute(("y", y_attr.as_str()));
        elem.push_attribute(("fill", fill.as_str()));
        elem.push_attribute(("font", self.state.font.as_str()));
        elem.push_attribute((
            "text-anchor",
            match self.state.text_align {
                TextAlign::Left | TextAlign::Start => "start",
                TextAlign::Center => "middle",
                TextAlign::Right | TextAlign::End => "end",
            },
        ));
        elem.push_attribute((
            "dominant-baseline",
            match self.state.text_baseline {
                TextBaseline::Top => "text-before-edge",
                TextBaseline::Hanging => "hanging",
                TextBaseline::Middle => "middle",
                TextBaseline::Alphabetic => "alphabetic",
                TextBaseline::Ideographic => "ideographic",
                TextBaseline::Bottom => "text-after-edge",
            },
        ));
        self.apply_transform_attr(&mut elem);
        self.writer.write_event(Event::Start(elem))?;
        self.writer.write_event(Event::Text(BytesText::new(text)))?;
        self.writer.write_event(Event::End(BytesEnd::new("text")))?;
        Ok(())
    }

    fn stroke_text(
        &mut self,
        _text: &str,
        _x: f64,
        _y: f64,
        _max_width: Option<f64>,
    ) -> Result<()> {
        Err(Self::not_supported("stroke_text"))
    }

    fn measure_text(&self, _text: &str) -> Result<TextMetrics> {
        Err(Self::not_supported("measure_text"))
    }
}

impl<W: Write> CanvasImageData for SvgCanvas<W> {
    fn create_image_data(&mut self, _width: u32, _height: u32) -> Result<ImageData> {
        Err(Self::not_supported("create_image_data"))
    }

    fn get_image_data(&self, _sx: u32, _sy: u32, _sw: u32, _sh: u32) -> Result<ImageData> {
        Err(Self::not_supported("get_image_data"))
    }

    fn put_image_data(&mut self, _data: &ImageData, _dx: f64, _dy: f64) -> Result<()> {
        Err(Self::not_supported("put_image_data"))
    }

    fn put_image_data_dirty(
        &mut self,
        _data: &ImageData,
        _dx: f64,
        _dy: f64,
        _dirty_x: u32,
        _dirty_y: u32,
        _dirty_width: u32,
        _dirty_height: u32,
    ) -> Result<()> {
        Err(Self::not_supported("put_image_data_dirty"))
    }
}

impl<W: Write> CanvasDrawImage for SvgCanvas<W> {
    fn draw_image(&mut self, image: &dyn CanvasImageSource, dx: f64, dy: f64) -> Result<()> {
        let href = self.encode_image_as_data_uri(image)?;
        let mut elem = BytesStart::new("image");
        let w_attr = image.width().to_string();
        let h_attr = image.height().to_string();
        let dx_attr = dx.to_string();
        let dy_attr = dy.to_string();
        elem.push_attribute(("x", dx_attr.as_str()));
        elem.push_attribute(("y", dy_attr.as_str()));
        elem.push_attribute(("width", w_attr.as_str()));
        elem.push_attribute(("height", h_attr.as_str()));
        elem.push_attribute(("href", href.as_str()));
        self.apply_transform_attr(&mut elem);
        self.write_empty(elem)
    }

    fn draw_image_scaled(
        &mut self,
        image: &dyn CanvasImageSource,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> Result<()> {
        let href = self.encode_image_as_data_uri(image)?;
        let mut elem = BytesStart::new("image");
        let dx_attr = dx.to_string();
        let dy_attr = dy.to_string();
        let dw_attr = dw.to_string();
        let dh_attr = dh.to_string();
        elem.push_attribute(("x", dx_attr.as_str()));
        elem.push_attribute(("y", dy_attr.as_str()));
        elem.push_attribute(("width", dw_attr.as_str()));
        elem.push_attribute(("height", dh_attr.as_str()));
        elem.push_attribute(("href", href.as_str()));
        elem.push_attribute(("preserveAspectRatio", "none"));
        self.apply_transform_attr(&mut elem);
        self.write_empty(elem)
    }

    fn draw_image_subrect(
        &mut self,
        _image: &dyn CanvasImageSource,
        _sx: f64,
        _sy: f64,
        _sw: f64,
        _sh: f64,
        _dx: f64,
        _dy: f64,
        _dw: f64,
        _dh: f64,
    ) -> Result<()> {
        Err(Self::not_supported("draw_image_subrect"))
    }
}

impl<W: Write> CanvasRenderingContext2D for SvgCanvas<W> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{
        CanvasDrawImage, CanvasFillStrokeStyles, CanvasRectangles, CanvasTransforms, ImageData,
        Paint, PatternRepetition,
    };

    fn svg_output<F>(f: F) -> String
    where
        F: FnOnce(&mut SvgCanvas<Vec<u8>>) -> Result<()>,
    {
        let buf = Vec::new();
        let mut svg = SvgCanvas::new(buf, 100.0, 100.0).expect("create svg");
        f(&mut svg).expect("draw operations");
        let out = svg.finish().expect("finish svg");
        String::from_utf8(out).expect("utf8")
    }

    #[test]
    fn writes_rect_fill() {
        let out = svg_output(|svg| {
            svg.set_fill_style(Paint::Color("red".into()))?;
            svg.fill_rect(0.0, 0.0, 10.0, 10.0)
        });

        assert!(out.contains("<rect x=\"0\" y=\"0\" width=\"10\" height=\"10\" fill=\"red\"/>"));
    }

    #[test]
    fn writes_linear_gradient_defs_and_usage() {
        let out = svg_output(|svg| {
            let mut grad = svg.create_linear_gradient(0.0, 0.0, 10.0, 0.0)?;
            grad.add_color_stop(0.0, "red");
            grad.add_color_stop(1.0, "blue");
            svg.set_fill_style(Paint::Gradient(grad))?;
            svg.fill_rect(0.0, 0.0, 10.0, 10.0)
        });

        assert!(out.contains("<linearGradient id=\"grad0\" x1=\"0\" y1=\"0\" x2=\"10\" y2=\"0\""));
        assert!(out.contains("<stop offset=\"0\" stop-color=\"red\"/>"));
        assert!(out.contains("<stop offset=\"1\" stop-color=\"blue\"/>"));
        assert!(out.contains("fill=\"url(#grad0)\""));
    }

    #[test]
    fn applies_transform_to_rect() {
        let out = svg_output(|svg| {
            svg.translate(5.0, 6.0)?;
            svg.set_fill_style(Paint::Color("black".into()))?;
            svg.fill_rect(0.0, 0.0, 4.0, 4.0)
        });

        assert!(out.contains("transform=\"matrix(1 0 0 1 5 6)\""));
    }

    #[test]
    fn writes_pattern_defs_and_usage() {
        let out = svg_output(|svg| {
            let pat = svg.create_pattern(&DummyImage, PatternRepetition::Repeat)?;
            svg.set_fill_style(Paint::Pattern(pat))?;
            svg.fill_rect(0.0, 0.0, 5.0, 5.0)
        });

        assert!(out.contains("<pattern id=\"pat0\""));
        assert!(out.contains("fill=\"url(#pat0)\""));
    }

    #[test]
    fn draw_image_inlines_png_data_uri() {
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![255, 0, 0, 255],
        };
        let out = svg_output(|svg| svg.draw_image(&img, 2.0, 3.0));

        assert!(out.contains("<image"));
        assert!(out.contains("x=\"2\" y=\"3\" width=\"1\" height=\"1\""));
        assert!(out.contains("href=\"data:image/png;base64,"));
    }

    struct DummyImage;
    impl CanvasImageSource for DummyImage {
        fn width(&self) -> u32 {
            1
        }

        fn height(&self) -> u32 {
            1
        }

        fn data_rgba(&self) -> Option<&[u8]> {
            None
        }
    }
}
