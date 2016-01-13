#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate x11;

use std::ptr;
use x11::xlib::*;

struct WindowManager {
    display: *mut Display,
    sw: u32,
    sh: u32,
    clients: Vec<Client>,
}

struct Client {
    win: Window,
    is_above: bool,
}

impl WindowManager {
    fn new() -> Self {
        unsafe {
            let display = XOpenDisplay(ptr::null_mut());
            let screen = XDefaultScreen(display);
            let sw = XDisplayWidth(display, screen) as u32;
            let sh = XDisplayHeight(display, screen) as u32;
            let root = XRootWindow(display, screen);

            XSelectInput(display, root, SubstructureRedirectMask | SubstructureNotifyMask );

            WindowManager {
                display: display,
                sw: sw,
                sh: sh,
                clients: vec![],
            }
        }
    }

    #[allow(non_upper_case_globals)]
    fn run_once(&mut self) {
        let mut ev = XEvent { pad : [0; 24] };
        let result = unsafe { XNextEvent(self.display, &mut ev) };
        if result == 0 {
            match ev.get_type() {
                MapRequest => {
                    self.handle_map_request(From::from(ev));
                }
                ConfigureRequest => {
                    self.handle_configure_request(From::from(ev));
                }
                UnmapNotify => {
                    self.handle_unmap_notify(From::from(ev));
                }
                _ => {
                    debug!("event: {:?}", ev.get_type());
                }
            }
        }
    }

    fn sync(&mut self) {
        unsafe {
            XSync(self.display, False);
        }
    }

    fn run(&mut self) {
        loop {
            self.run_once();
        }
    }

    fn arrange_clients(&mut self) {
        for c in &self.clients {
            unsafe {
                if c.is_above {
                    XRaiseWindow(self.display, c.win);
                } else {
                    XLowerWindow(self.display, c.win);
                }
                XMoveResizeWindow(self.display, c.win, 0, 0, self.sw, self.sh);
            }
        }
    }

    fn handle_map_request(&mut self, ev: XMapRequestEvent) {
        debug!("[{}] MapRequest", ev.window);
        let client = Client {
            win: ev.window,
            is_above: false,
        };
        self.clients.push(client);
        unsafe {
            XMapWindow(self.display, ev.window);
        }
        self.arrange_clients();
    }

    fn handle_configure_request(&mut self, ev: XConfigureRequestEvent) {
        debug!("[{}] ConfigureRequest", ev.window);

        let mut wc = XWindowChanges {
            x: ev.x,
            y: ev.y,
            width: ev.width,
            height: ev.height,
            border_width: ev.border_width,
            sibling: ev.above,
            stack_mode: ev.detail,
        };
        unsafe {
            XConfigureWindow(self.display, ev.window, ev.value_mask as u32, &mut wc);
        }
        self.sync();
    }

    fn handle_unmap_notify(&mut self, ev: XConfigureRequestEvent) {
        debug!("[{}] UnmapNotify", ev.window);
        self.clients.retain(|c| c.win != ev.window);
    }
}

fn main() {
    env_logger::init().unwrap();
    let mut wm = WindowManager::new();
    wm.run();    
}
