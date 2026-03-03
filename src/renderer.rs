use crate::theme::Theme;
use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics,
    Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub cache: Cache,
    pub viewport: Viewport,
    pub text_atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub theme: Theme,
}

impl Renderer {
    pub fn new(
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Self {
        let swapchain_format = surface
            .get_capabilities(adapter)
            .formats
            .first()
            .copied()
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let mut font_system = FontSystem::new();
        font_system
            .db_mut()
            .load_font_data(include_bytes!("../assets/JetBrainsMono-Regular.ttf").to_vec());

        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut text_atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
        let text_renderer = TextRenderer::new(
            &mut text_atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        );

        Self {
            device,
            queue,
            surface,
            surface_config,
            font_system,
            swash_cache,
            cache,
            viewport,
            text_atlas,
            text_renderer,
            theme: Theme::default(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn width(&self) -> u32 {
        self.surface_config.width
    }

    pub fn height(&self) -> u32 {
        self.surface_config.height
    }

    pub fn render(&mut self, text_areas: &[TextArea<'_>]) -> Result<(), wgpu::SurfaceError> {
        self.viewport.update(
            &self.queue,
            Resolution {
                width: self.surface_config.width,
                height: self.surface_config.height,
            },
        );

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .expect("text prepare failed");

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let bg = &self.theme.bg;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: bg[0],
                            g: bg[1],
                            b: bg[2],
                            a: bg[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.text_renderer
                .render(&self.text_atlas, &self.viewport, &mut pass)
                .expect("text render failed");
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        self.text_atlas.trim();

        Ok(())
    }

    pub fn create_text_buffer(
        &mut self,
        text: &str,
        width: f32,
        color: [f32; 4],
    ) -> GlyphonBuffer {
        let metrics = Metrics::new(self.theme.font_size, self.theme.line_height_px());
        let mut buffer = GlyphonBuffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(width), None);
        buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new().family(Family::Monospace).color(GlyphonColor::rgba(
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
                (color[3] * 255.0) as u8,
            )),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
    }

    /// Measure the width of a single character in the current monospace font.
    pub fn char_width(&mut self) -> f32 {
        let metrics = Metrics::new(self.theme.font_size, self.theme.line_height_px());
        let mut buffer = GlyphonBuffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(500.0), None);
        buffer.set_text(
            &mut self.font_system,
            "M",
            &Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                return glyph.w;
            }
        }
        self.theme.font_size * 0.6
    }
}
