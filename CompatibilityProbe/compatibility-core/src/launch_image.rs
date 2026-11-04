//! Texture mapping for the static, top-down launch bitmap only.
use crate::matrix::Matrix;

pub fn texture_matrix(rotation: Matrix<2>) -> Matrix<2> {
    // Matrix::multiply applies its receiver first and its argument second.
    // Reflect the output quad before rotating into the portrait bitmap. In
    // landscape, reversing this order turns Peggle's pre-rotated logo 180°.
    Matrix::y_flip().multiply(&rotation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;
    fn assert_point(actual: [f32; 2], expected: [f32; 2]) {
        for i in 0..2 { assert!((actual[i]-expected[i]).abs()<0.00001, "{actual:?} != {expected:?}"); }
    }
    #[test]
    fn phone_launch_artwork_quarter_turn_maps_all_four_corners_upright() {
        // Peggle's Default.png is portrait with its logo rotated clockwise.
        // A screen's TL/TR/BL/BR must sample bitmap TR/BR/TL/BL respectively.
        let m=texture_matrix(Matrix::z_rotation(FRAC_PI_2));
        for (screen,bitmap) in [([-1.,1.],[1.,-1.]),([1.,1.],[1.,1.]),([-1.,-1.],[-1.,-1.]),([1.,-1.],[-1.,1.])] {
            assert_point(m.transform(screen),bitmap);
        }
    }
    #[test]
    fn opposite_landscape_uses_the_opposite_quarter_turn() {
        let m=texture_matrix(Matrix::z_rotation(-FRAC_PI_2));
        for (screen,bitmap) in [([-1.,1.],[-1.,1.]),([1.,1.],[-1.,-1.]),([-1.,-1.],[1.,1.]),([1.,-1.],[1.,-1.])] {
            assert_point(m.transform(screen),bitmap);
        }
    }
    #[test]
    fn portrait_keeps_the_original_top_down_row_correction() {
        let m=texture_matrix(Matrix::identity());
        assert_point(m.transform([-1.,1.]),[-1.,-1.]);
        assert_point(m.transform([1.,-1.]),[1.,1.]);
    }
}

/// Verified legacy launch resources: UIKit reports dimensions in points,
/// while these files contain either one or two pixels per point. This narrow
/// compatibility rule also identifies the pre-rotated phone launch artwork.
/// HD and all other UIImage resources keep their existing handling.
pub fn guest_launch_scale(bundle_id: &str, version: &str, resource: &str,
    dimensions: (u32, u32)) -> Option<f32> {
    if bundle_id != "com.popcap.Peggle" || version != "4824.14.07.16.01" { return None; }
    match (resource, dimensions) {
        ("Default.png", (320,480)) => Some(1.0),
        ("Default@2x.png", (640,960)) | ("Default-568h@2x.png", (640,1136)) => Some(2.0),
        _ => None,
    }
}

pub fn half_turn_rgba(pixels: &[u8]) -> Vec<u8> {
    assert_eq!(pixels.len() % 4, 0);
    pixels.chunks_exact(4).rev().flat_map(|pixel| pixel.iter().copied()).collect()
}

#[cfg(test)]
mod guest_tests {
    use super::*;
    #[test]
    fn half_turn_preserves_channels_and_reverses_both_axes() {
        let pixels: Vec<u8> = (0..24).collect(); // 3 columns, 2 rows
        let rotated = half_turn_rgba(&pixels);
        assert_eq!(rotated, vec![20,21,22,23,16,17,18,19,12,13,14,15,8,9,10,11,4,5,6,7,0,1,2,3]);
        assert_eq!(half_turn_rgba(&rotated), pixels);
    }
    #[test]
    fn correction_is_limited_to_verified_phone_launch_resources() {
        let check = |version, resource, dimensions| guest_launch_scale("com.popcap.Peggle", version, resource, dimensions);
        assert_eq!(check("4824.14.07.16.01", "Default.png", (320,480)), Some(1.0));
        assert_eq!(check("4824.14.07.16.01", "Default@2x.png", (640,960)), Some(2.0));
        assert_eq!(check("4824.14.07.16.01", "Default-568h@2x.png", (640,1136)), Some(2.0));
        assert_eq!(check("2.0.9", "Default.png", (1024,768)), None);
        assert_eq!(check("4824.14.07.16.01", "game.png", (320,480)), None);
        assert_eq!(check("4824.14.07.16.01", "Default.png", (640,960)), None);
        assert_eq!(guest_launch_scale("another.app", "4824.14.07.16.01", "Default.png", (320,480)), None);
    }
}
