use gpui::TestAppContext;

pub fn init_test(cx: &mut TestAppContext) {
    // Add things we need across the app for testing
    cx.update(|_cx| {
        // mutable App for initializing things
    });
}
