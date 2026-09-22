// Lighting Effects Engine
// Generates RGB lighting effects with animations and transitions

use crate::rgb::{RgbColor, RgbEffect};
use std::collections::HashMap;
use std::time::Instant;
use serde::{Deserialize, Serialize};

/// Effect context for calculations
#[derive(Debug, Clone)]
pub struct EffectContext {
    pub frame: u64,
    pub total_zones: u32,
    pub active_zone: Option<u32>,
}

/// Effect configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectConfig {
    pub name: String,
    pub speed: u8, // 1-10 (1 = slowest)
    pub color: RgbColor,
    pub brightness: u8, // 0-100
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            speed: 5,
            color: RgbColor { red: 0, green: 255, blue: 255 }, // Cyan
            brightness: 100,
        }
    }
}

/// Base trait for all effects
pub trait LightingEffect: Send {
    /// Get effect name
    fn name(&self) -> &str;
    
    /// Calculate color for specific zone at current time
    fn get_zone_color(&self, ctx: &EffectContext) -> RgbColor;
    
    /// Update internal state
    fn update(&mut self, delta_ms: u64);
    
    /// Check if effect is complete (for one-shot effects)
    fn is_complete(&self) -> bool {
        false
    }
}

/// Static single color effect
pub struct StaticEffect {
    config: EffectConfig,
    color: RgbColor,
}

impl StaticEffect {
    pub fn new(color: RgbColor) -> Self {
        Self {
            config: EffectConfig::default(),
            color,
        }
    }
}

impl LightingEffect for StaticEffect {
    fn name(&self) -> &str {
        "Static"
    }
    
    fn get_zone_color(&self, _ctx: &EffectContext) -> RgbColor {
        self.color
    }
    
    fn update(&mut self, _delta_ms: u64) {}
}

/// Breathing effect (fade in/out)
pub struct BreathingEffect {
    config: EffectConfig,
    phase: f32, // 0.0 - 2π
    direction: i8, // 1 = brightening, -1 = darkening
}

impl BreathingEffect {
    pub fn new(_color: RgbColor) -> Self {
        Self {
            config: EffectConfig::default(),
            phase: 0.0,
            direction: 1,
        }
    }
    
    fn calculate_intensity(&self, ctx: &EffectContext) -> f32 {
        // Different breathing patterns for different zones
        let zone_offset = ctx.active_zone.unwrap_or(0) as f32 * 0.5;
        let t = self.phase + zone_offset;
        (t.sin() + 1.0) / 2.0 // Convert to 0-1 range
    }
}

impl LightingEffect for BreathingEffect {
    fn name(&self) -> &str {
        "Breathing"
    }
    
    fn get_zone_color(&self, ctx: &EffectContext) -> RgbColor {
        let intensity = self.calculate_intensity(ctx);
        
        RgbColor {
            red: (self.config.color.red as f32 * intensity) as u8,
            green: (self.config.color.green as f32 * intensity) as u8,
            blue: (self.config.color.blue as f32 * intensity) as u8,
        }
    }
    
    fn update(&mut self, delta_ms: u64) {
        // Speed affects how much phase advances per frame
        let speed_factor = self.config.speed as f32 / 5.0;
        let phase_increment = (delta_ms as f32 * 0.001) * speed_factor;
        
        self.phase += phase_increment;
        if self.phase > std::f32::consts::PI * 2.0 {
            self.phase -= std::f32::consts::PI * 2.0;
        }
    }
}

/// Spectrum/Rainbow effect
pub struct SpectrumEffect {
    base_color: RgbColor,
    hue_offset: u16,
    config: EffectConfig,  // Added for consistency
}

impl SpectrumEffect {
    pub fn new(base_color: RgbColor) -> Self {
        Self {
            base_color,
            hue_offset: 0,
            config: EffectConfig::default(),
        }
    }
    
    fn hsv_to_rgb_shifted(hue: u16) -> RgbColor {
        let hue_f = hue as f32 / 255.0 * 6.0;
        let sector = hue_f as i32 % 6;
        let frac = hue_f - hue_f as i32 as f32;
        let v = 255;
        
        let p = (v as f32 * (255.0 - 255.0 * frac) / 255.0) as u8;
        let q = (v as f32 * (255.0 - 255.0 * frac * 0.6) / 255.0) as u8;
        let t = (v as f32 * (255.0 - 255.0 * (1.0 - frac)) / 255.0) as u8;
        
        match sector {
            0 => RgbColor { red: v, green: t, blue: p },
            1 => RgbColor { red: q, green: v, blue: p },
            2 => RgbColor { red: p, green: v, blue: t },
            3 => RgbColor { red: p, green: q, blue: v },
            4 => RgbColor { red: t, green: p, blue: v },
            _ => RgbColor { red: v, green: p, blue: q },
        }
    }
}

impl LightingEffect for SpectrumEffect {
    fn name(&self) -> &str {
        "Spectrum"
    }
    
    fn get_zone_color(&self, ctx: &EffectContext) -> RgbColor {
        let zone = ctx.active_zone.unwrap_or(0);
        let shifted_hue = (self.hue_offset + (zone * 10) as u16) % 256;
        Self::hsv_to_rgb_shifted(shifted_hue)
    }
    
    fn update(&mut self, delta_ms: u64) {
        let speed_factor = 5.0 / self.config.speed as f32;
        let increment = ((delta_ms as f32 / (speed_factor * 50.0)).clamp(1.0, 10.0) as u32).max(1);
        self.hue_offset = (self.hue_offset as u32 + increment).min(10) as u16 % 256;
    }
}

/// Wave effect (moving pattern across zones)
pub struct WaveEffect {
    wave_pos: u32,
    wave_length: u32,
    direction: i8, // 1 or -1
    colors: Vec<RgbColor>,
    wave_position: usize,
    config: EffectConfig,  // Added for consistency
}

impl WaveEffect {
    pub fn new(colors: Vec<RgbColor>) -> Self {
        Self {
            wave_pos: 0,
            wave_length: 20, // pixels wide
            direction: 1,
            colors,
            wave_position: 0,
            config: EffectConfig::default(),
        }
    }
    
    fn interpolate_color(&self, factor: f32) -> RgbColor {
        // Interpolate between two colors based on factor (0.0-1.0)
        let c1 = &self.colors[self.wave_position];
        let c2_index = (self.wave_position + 1) % self.colors.len();
        let c2 = &self.colors[c2_index];
        
        RgbColor {
            red: ((c1.red as f32 * (1.0 - factor) + c2.red as f32 * factor) as u8),
            green: ((c1.green as f32 * (1.0 - factor) + c2.green as f32 * factor) as u8),
            blue: ((c1.blue as f32 * (1.0 - factor) + c2.blue as f32 * factor) as u8),
        }
    }
    
    fn calculate_wave_intensity(&self, zone_id: u32) -> f32 {
        let distance = ((zone_id as i64) - (self.wave_pos as i64)).abs();
        
        if (distance as u32) < self.wave_length {
            (1.0 - (distance as f32 / self.wave_length as f32)).powi(2)
        } else {
            0.0
        }
    }
}

impl LightingEffect for WaveEffect {
    fn name(&self) -> &str {
        "Wave"
    }
    
    fn get_zone_color(&self, ctx: &EffectContext) -> RgbColor {
        let zone_id = ctx.active_zone.unwrap_or(0);
        let intensity = self.calculate_wave_intensity(zone_id);
        
        if intensity < 0.1 {
            return RgbColor { red: 0, green: 0, blue: 0 };
        }
        
        let position_in_wave = 1.0 - (intensity / intensity.max(0.1));
        self.interpolate_color(position_in_wave)
    }
    
    fn update(&mut self, delta_ms: u64) {
        let speed_multiplier = 1u64.max(self.config.speed as u64);
        let move_steps = (delta_ms / (1000u64.saturating_mul(10).saturating_div(speed_multiplier))).min(20);
        
        for _ in 0..move_steps {
            self.wave_pos = self.wave_pos.wrapping_add(self.direction as u32);
            self.wave_position = if self.direction > 0 {
                (self.wave_position + 1) % self.colors.len()
            } else {
                self.wave_position.checked_sub(1).unwrap_or(0)
            };
        }
    }
}

/// Reactive effect (lights up on key/button press)
pub struct ReactiveEffect {
    trigger_zones: Vec<u32>,
    decay_timer: u64, // ms until effect ends
    decay_rate: u64,  // ms per tick
    active_zones: Vec<(u32, u64)>, // (zone, remaining_time)
}

impl ReactiveEffect {
    pub fn new(trigger_zones: Vec<u32>) -> Self {
        Self {
            trigger_zones,
            decay_timer: 200, // 200ms duration
            decay_rate: 10,   // check every 10ms
            active_zones: vec![],
        }
    }
    
    pub fn trigger_zone(&mut self, zone: u32) {
        if !self.active_zones.iter().any(|(z, _)| *z == zone) {
            self.active_zones.push((zone, self.decay_timer));
        }
    }
}

impl LightingEffect for ReactiveEffect {
    fn name(&self) -> &str {
        "Reactive"
    }
    
    fn get_zone_color(&self, ctx: &EffectContext) -> RgbColor {
        let zone_id = ctx.active_zone.unwrap_or(0);
        
        // Find this zone's remaining time
        if let Some((_, remaining)) = self.active_zones.iter().find(|(z, _)| *z == zone_id) {
            let intensity = *remaining as f32 / self.decay_timer as f32;
            
            // Green to black fade
            RgbColor {
                red: (0 as f32 * intensity) as u8,
                green: (255 as f32 * intensity) as u8,
                blue: (0 as f32 * intensity) as u8,
            }
        } else {
            RgbColor { red: 0, green: 0, blue: 0 }
        }
    }
    
    fn update(&mut self, delta_ms: u64) {
        // Decay all active zones
        self.active_zones.retain_mut(|(_, remaining)| {
            *remaining = remaining.saturating_sub(delta_ms);
            *remaining > 0
        });
    }
    
    fn is_complete(&self) -> bool {
        self.active_zones.is_empty()
    }
}

/// Factory for creating effects
pub struct EffectFactory;

impl EffectFactory {
    pub fn create(effect_type: &RgbEffect) -> Box<dyn LightingEffect> {
        match effect_type {
            RgbEffect::Static => Box::new(StaticEffect::new(RgbColor {
                red: 0, green: 255, blue: 255
            })),
            RgbEffect::Breathing => Box::new(BreathingEffect::new(RgbColor {
                red: 0, green: 255, blue: 255
            })),
            RgbEffect::Spectrum => Box::new(SpectrumEffect::new(RgbColor {
                red: 0, green: 255, blue: 255
            })),
            RgbEffect::Wave(_) => {
                let colors = vec![
                    RgbColor { red: 255, green: 0, blue: 0 },
                    RgbColor { red: 255, green: 128, blue: 0 },
                    RgbColor { red: 255, green: 255, blue: 0 },
                    RgbColor { red: 0, green: 255, blue: 0 },
                    RgbColor { red: 0, green: 255, blue: 255 },
                    RgbColor { red: 0, green: 0, blue: 255 },
                    RgbColor { red: 255, green: 0, blue: 255 },
                ];
                Box::new(WaveEffect::new(colors))
            }
            RgbEffect::Reactive => Box::new(ReactiveEffect::new(vec![0])),
            RgbEffect::Gradient(_) => {
                // Similar to spectrum for now
                Box::new(SpectrumEffect::new(RgbColor {
                    red: 0, green: 255, blue: 255
                }))
            }
            RgbEffect::Custom => Box::new(StaticEffect::new(RgbColor::default())),
            RgbEffect::Off => Box::new(StaticEffect::new(RgbColor {
                red: 0, green: 0, blue: 0
            })),
        }
    }
}

/// Animation manager
pub struct AnimationManager {
    effects: HashMap<String, Box<dyn LightingEffect>>,
    current_effect_name: Option<String>,
    last_update: Instant,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
            current_effect_name: None,
            last_update: Instant::now(),
        }
    }
    
    pub fn register_effect(&mut self, name: &str, effect: Box<dyn LightingEffect>) {
        self.effects.insert(name.to_string(), effect);
    }
    
    pub fn set_effect(&mut self, effect_type: &RgbEffect) -> anyhow::Result<()> {
        let effect_name = effect_type_to_str(effect_type);
        let new_effect = EffectFactory::create(effect_type);
        
        self.register_effect(&effect_name, new_effect);
        self.current_effect_name = Some(effect_name);
        
        Ok(())
    }
    
    pub fn get_current_color(&self, ctx: &EffectContext) -> Option<RgbColor> {
        self.current_effect_name.as_ref().and_then(|name| {
            self.effects.get(name).map(|effect| effect.get_zone_color(ctx))
        })
    }
    
    pub fn update(&mut self, elapsed_ms: u64) {
        if let Some(ref name) = self.current_effect_name {
            if let Some(effect) = self.effects.get_mut(name) {
                effect.update(elapsed_ms);
            }
        }
        self.last_update = Instant::now();
    }
}

fn effect_type_to_str(effect: &RgbEffect) -> String {
    match effect {
        RgbEffect::Static => "static",
        RgbEffect::Breathing => "breathing",
        RgbEffect::Spectrum => "spectrum",
        RgbEffect::Wave(_) => "wave",
        RgbEffect::Reactive => "reactive",
        RgbEffect::Gradient(_) => "gradient",
        RgbEffect::Custom => "custom",
        RgbEffect::Off => "off",
    }.to_string()
}
