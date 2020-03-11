use minterpolate::InterpolationPrimitive;
use serde::{Deserialize, Serialize};

use amethyst_assets::Handle;
use amethyst_core::math::zero;
use amethyst_rendy::light::Light;

use crate::{util::SamplerPrimitive, AnimationSampling, ApplyData, BlendMethod};
/// Channels that can be animated on `Light`
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum LightChannel {
    Color,
}
impl<'a> ApplyData<'a> for Light {
    type ApplyData = ();
}
impl AnimationSampling for Light {
    type Primitive = SamplerPrimitive<f32>;
    type Channel = LightChannel;

    fn apply_sample(&mut self, channel: &Self::Channel, data: &SamplerPrimitive<f32>, _: &()) {
        use self::LightChannel::*;
        use crate::util::SamplerPrimitive::*;
        match self {
            Light::Point(point_light) => match (channel, *data) {
                (&Color, Vec3(ref d)) => {
                    point_light.color.red = d[0];
                    point_light.color.green = d[1];
                    point_light.color.blue = d[2];
                }
                _ => panic!("Attempt to apply to wrong Light type"),
            },
            _ => panic!("Attempt to apply invalid sample to Light"),
        }
    }

    fn current_sample(&self, channel: &Self::Channel, _: &()) -> SamplerPrimitive<f32> {
        use self::LightChannel::*;
        match (self, channel) {
            (Light::Point(point_light), Color) => SamplerPrimitive::Vec3([
                point_light.color.red,
                point_light.color.green,
                point_light.color.blue,
            ])
            .into(),
            _ => panic!("wrong light type"),
        }
    }
    fn default_primitive(channel: &Self::Channel) -> Self::Primitive {
        use self::LightChannel::*;
        match channel {
            Color => SamplerPrimitive::Vec3([zero(); 3]),
        }
    }

    fn blend_method(&self, _: &Self::Channel) -> Option<BlendMethod> {
        Some(BlendMethod::Linear)
    }
}
