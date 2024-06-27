pub struct NotSendNorSync {
    pointer: *const (),
}

pub struct SendAndSync {
    x: usize,
}

pub struct SendNotSync {
    x: std::cell::Cell<usize>,
}
