use warp::reply::{Reply, Response};

// Implemented for all types that implement `Reply`.
//
// A user doesn't need to worry about this, it's just trait
// hackery to get `Box<dyn Reply>` working.
pub trait BoxedReply: Reply {
    fn boxed_into_response(self: Box<Self>) -> Response;
}

impl<T: Reply> BoxedReply for T {
    fn boxed_into_response(self: Box<Self>) -> Response {
        self.into_response()
    }
}

#[allow(missing_debug_implementations)]
pub struct DynReply(Box<dyn BoxedReply>);

impl Reply for DynReply {
    fn into_response(self) -> Response {
        self.0.boxed_into_response()
    }
}

// Workaround for missing Reply implementation of Box<dyn Reply>
pub fn dyn_reply<T: Reply + 'static>(reply: T) -> DynReply {
    DynReply(Box::new(reply))
}
