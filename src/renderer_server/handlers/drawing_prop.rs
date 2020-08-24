use tokio::sync::oneshot;

use crate::ipc_protocol::{ServerOneshotSender, ServerResponse, DrawingProp, DrawingPropValue};

use super::HandlerError;
use super::super::{
    event_loop_notifier::EventLoopNotifier,
    state::DrawingState,
    access_control::AccessControl,
};

pub(crate) async fn drawing_prop(
    data_req_queued: oneshot::Sender<()>,
    conn: ServerOneshotSender,
    app_control: &AccessControl,
    prop: DrawingProp,
) -> Result<(), HandlerError> {
    let drawing = app_control.get_drawing(data_req_queued).await;
    let drawing = drawing.lock().await;

    use DrawingProp::*;
    let value = match prop {
        Title => DrawingPropValue::Title(drawing.title.clone()),
        Background => DrawingPropValue::Background(drawing.background),
        Center => DrawingPropValue::Center(drawing.center),
        Size => DrawingPropValue::Size(crate::Size {width: drawing.width, height: drawing.height}),
        Width => DrawingPropValue::Width(drawing.width),
        Height => DrawingPropValue::Height(drawing.height),
        IsMaximized => DrawingPropValue::IsMaximized(drawing.is_maximized),
        IsFullscreen => DrawingPropValue::IsFullscreen(drawing.is_fullscreen),
    };

    conn.send(ServerResponse::DrawingProp(value))?;

    Ok(())
}

pub(crate) async fn set_drawing_prop(
    data_req_queued: oneshot::Sender<()>,
    app_control: &AccessControl,
    event_loop: EventLoopNotifier,
    prop_value: DrawingPropValue,
) -> Result<(), HandlerError> {
    let drawing = app_control.get_drawing(data_req_queued).await;
    let mut drawing = drawing.lock().await;

    modify_drawing(&mut drawing, event_loop, prop_value).await
}

pub(crate) async fn reset_drawing_prop(
    data_req_queued: oneshot::Sender<()>,
    app_control: &AccessControl,
    event_loop: EventLoopNotifier,
    prop: DrawingProp,
) -> Result<(), HandlerError> {
    let drawing = app_control.get_drawing(data_req_queued).await;
    let mut drawing = drawing.lock().await;

    use DrawingProp::*;
    modify_drawing(&mut drawing, event_loop, match prop {
        Title => DrawingPropValue::Title(DrawingState::DEFAULT_TITLE.to_string()),
        Background => DrawingPropValue::Background(DrawingState::DEFAULT_BACKGROUND),
        Center => DrawingPropValue::Center(DrawingState::DEFAULT_CENTER),
        Size => DrawingPropValue::Size(crate::Size {
            width: DrawingState::DEFAULT_WIDTH,
            height: DrawingState::DEFAULT_HEIGHT,
        }),
        Width => DrawingPropValue::Width(DrawingState::DEFAULT_WIDTH),
        Height => DrawingPropValue::Height(DrawingState::DEFAULT_HEIGHT),
        IsMaximized => DrawingPropValue::IsMaximized(DrawingState::DEFAULT_IS_MAXIMIZED),
        IsFullscreen => DrawingPropValue::IsFullscreen(DrawingState::DEFAULT_IS_FULLSCREEN),
    }).await
}

async fn modify_drawing(
    drawing: &mut DrawingState,
    event_loop: EventLoopNotifier,
    prop_value: DrawingPropValue,
) -> Result<(), HandlerError> {
    use DrawingPropValue::*;
    match prop_value {
        Title(title) => {
            drawing.title = title.clone();

            // Signal the main thread to change this property on the window
            event_loop.set_title(title)?;
        },

        Background(background) => {
            drawing.background = background;

            // Signal the main thread that the image has changed
            event_loop.request_redraw()?;
        },

        Center(center) => {
            drawing.center = center;

            // Signal the main thread that the image has changed
            event_loop.request_redraw()?;
        },

        Size(crate::Size {width, height}) => {
            drawing.width = width;
            drawing.height = height;

            // Signal the main thread to change this property on the window
            event_loop.set_size((width, height))?;
        },

        Width(width) => {
            drawing.width = width;

            // Signal the main thread to change this property on the window
            event_loop.set_size((width, drawing.height))?;
        },

        Height(height) => {
            drawing.height = height;

            // Signal the main thread to change this property on the window
            event_loop.set_size((drawing.width, height))?;
        },

        IsMaximized(is_maximized) => {
            drawing.is_maximized = is_maximized;

            // Signal the main thread to change this property on the window
            event_loop.set_is_maximized(is_maximized)?;
        },

        IsFullscreen(is_fullscreen) => {
            drawing.is_fullscreen = is_fullscreen;

            // Signal the main thread to change this property on the window
            event_loop.set_is_fullscreen(is_fullscreen)?;
        },
    }

    Ok(())
}
