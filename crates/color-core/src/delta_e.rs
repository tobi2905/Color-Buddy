//! CIEDE2000 colour difference (Sharma, Wu & Dalal 2005), with kL = kC = kH = 1.

use crate::convert::Lab;

pub fn ciede2000(lab1: Lab, lab2: Lab) -> f64 {
    let (l1, a1, b1) = (lab1.l, lab1.a, lab1.b);
    let (l2, a2, b2) = (lab2.l, lab2.a, lab2.b);

    let c1 = a1.hypot(b1);
    let c2 = a2.hypot(b2);
    let c_bar = (c1 + c2) / 2.0;
    let c_bar7 = c_bar.powi(7);
    let g = 0.5 * (1.0 - (c_bar7 / (c_bar7 + 25f64.powi(7))).sqrt());

    let a1p = (1.0 + g) * a1;
    let a2p = (1.0 + g) * a2;
    let c1p = a1p.hypot(b1);
    let c2p = a2p.hypot(b2);

    let hue = |b: f64, ap: f64| {
        if b == 0.0 && ap == 0.0 {
            0.0
        } else {
            b.atan2(ap).to_degrees().rem_euclid(360.0)
        }
    };
    let h1p = hue(b1, a1p);
    let h2p = hue(b2, a2p);

    let dl = l2 - l1;
    let dc = c2p - c1p;
    let dh = if c1p * c2p == 0.0 {
        0.0
    } else {
        let d = h2p - h1p;
        if d > 180.0 {
            d - 360.0
        } else if d < -180.0 {
            d + 360.0
        } else {
            d
        }
    };
    let dhh = 2.0 * (c1p * c2p).sqrt() * (dh / 2.0).to_radians().sin();

    let l_bar = (l1 + l2) / 2.0;
    let c_bar_p = (c1p + c2p) / 2.0;
    let h_bar_p = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        (h1p + h2p) / 2.0
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) / 2.0
    } else {
        (h1p + h2p - 360.0) / 2.0
    };

    let t = 1.0 - 0.17 * (h_bar_p - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_bar_p).to_radians().cos()
        + 0.32 * (3.0 * h_bar_p + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_bar_p - 63.0).to_radians().cos();
    let d_theta = 30.0 * (-((h_bar_p - 275.0) / 25.0).powi(2)).exp();
    let c_bar_p7 = c_bar_p.powi(7);
    let rc = 2.0 * (c_bar_p7 / (c_bar_p7 + 25f64.powi(7))).sqrt();
    let l50 = (l_bar - 50.0).powi(2);
    let sl = 1.0 + 0.015 * l50 / (20.0 + l50).sqrt();
    let sc = 1.0 + 0.045 * c_bar_p;
    let sh = 1.0 + 0.015 * c_bar_p * t;
    let rt = -(2.0 * d_theta).to_radians().sin() * rc;

    let tl = dl / sl;
    let tc = dc / sc;
    let th = dhh / sh;
    (tl * tl + tc * tc + th * th + rt * tc * th).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_colours_have_zero_difference() {
        let lab = Lab {
            l: 50.0,
            a: 10.0,
            b: -20.0,
        };
        assert_eq!(ciede2000(lab, lab), 0.0);
    }

    #[test]
    fn is_symmetric() {
        let a = Lab {
            l: 50.0,
            a: 2.6772,
            b: -79.7751,
        };
        let b = Lab {
            l: 50.0,
            a: 0.0,
            b: -82.7485,
        };
        assert!((ciede2000(a, b) - ciede2000(b, a)).abs() < 1e-12);
    }
}
