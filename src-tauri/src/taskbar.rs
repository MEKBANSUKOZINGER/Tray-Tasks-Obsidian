//! Dock a small window alongside the primary Windows notification area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
impl Rect {
    pub fn width(self) -> i32 {
        self.right - self.left
    }
    pub fn height(self) -> i32 {
        self.bottom - self.top
    }
    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}
pub fn dock(
    bar: Rect,
    notification: Option<Rect>,
    size: [u32; 2],
    gap: i32,
    fallback_x: i32,
) -> [i32; 2] {
    let w = size[0] as i32;
    let h = size[1] as i32;
    if bar.width() >= bar.height() {
        let end = notification
            .filter(|r| r.width() > 0 && r.left > bar.left && r.left < bar.right)
            .map(|r| r.left - gap);
        let x = end
            .map(|end| end - w)
            .unwrap_or(fallback_x)
            .clamp(bar.left, (bar.right - w).max(bar.left));
        [x, bar.top + (bar.height() - h) / 2]
    } else {
        // Vertical taskbars have too little width; keep the widget beside them.
        let y = notification
            .map(|r| r.top - gap - h)
            .unwrap_or(bar.bottom - h - gap)
            .clamp(bar.top, (bar.bottom - h).max(bar.top));
        [
            if bar.left <= 0 {
                bar.right + gap
            } else {
                bar.left - w - gap
            },
            y,
        ]
    }
}
#[cfg(windows)]
mod native {
    use super::Rect;
    use windows_sys::Win32::{
        Foundation::{HWND, POINT, RECT},
        UI::WindowsAndMessaging::{
            FindWindowExW, FindWindowW, GetCursorPos, GetWindowRect, SetWindowPos, HWND_TOPMOST,
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        },
    };
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    unsafe fn rect(hwnd: HWND) -> Option<Rect> {
        let mut r = RECT::default();
        if hwnd.is_null()
            || GetWindowRect(hwnd, &mut r) == 0
            || r.right <= r.left
            || r.bottom <= r.top
        {
            return None;
        }
        Some(Rect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        })
    }
    pub fn taskbar() -> Option<(Rect, Option<Rect>)> {
        // Shell window classes are an optional hint, never a required dependency.
        unsafe {
            let bar = FindWindowW(wide("Shell_TrayWnd").as_ptr(), std::ptr::null());
            let bounds = rect(bar)?;
            let notify = FindWindowExW(
                bar,
                std::ptr::null_mut(),
                wide("TrayNotifyWnd").as_ptr(),
                std::ptr::null(),
            );
            Some((bounds, rect(notify)))
        }
    }
    pub fn keep_above(hwnd: isize) {
        unsafe {
            SetWindowPos(
                hwnd as HWND,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            );
        }
    }
    pub fn cursor() -> Option<[i32; 2]> {
        let mut p = POINT::default();
        if unsafe { GetCursorPos(&mut p) } == 0 {
            None
        } else {
            Some([p.x, p.y])
        }
    }
}
#[cfg(windows)]
pub use native::{cursor, keep_above, taskbar};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notification_area_dock_stays_left_and_vertically_centered() {
        let bar = Rect {
            left: 0,
            top: 1392,
            right: 2560,
            bottom: 1440,
        };
        let notification = Rect {
            left: 2238,
            top: 1392,
            right: 2560,
            bottom: 1440,
        };
        assert_eq!(
            dock(bar, Some(notification), [280, 40], 8, 1950),
            [1950, 1396]
        );
    }
    #[test]
    fn fallback_ignores_old_y_and_clamps_x_to_current_screen() {
        let bar = Rect {
            left: -1920,
            top: 1040,
            right: 0,
            bottom: 1080,
        };
        assert_eq!(dock(bar, None, [280, 40], 8, 1950), [-280, 1040]);
        assert_eq!(dock(bar, None, [280, 40], 8, -500), [-500, 1040]);
        assert_eq!(
            dock(
                bar,
                Some(Rect {
                    left: -1920,
                    top: 1040,
                    right: 0,
                    bottom: 1080
                }),
                [280, 40],
                8,
                -500
            ),
            [-500, 1040]
        );
    }
    #[test]
    fn top_and_vertical_taskbars_use_their_own_bounds() {
        let top = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 48,
        };
        assert_eq!(dock(top, None, [280, 40], 8, 1000), [1000, 4]);
        let side = Rect {
            left: 0,
            top: 0,
            right: 48,
            bottom: 1080,
        };
        assert_eq!(dock(side, None, [280, 40], 8, 1000), [56, 1032]);
    }
}
