# TODO
20250104    [X] rewrite shape & layout, i need to preserve the shape's size etc
20250104    [X] fix cursor to stick only to the currently clicked object, to avoid hover collision
20250106    [X] function callback using `CallBack(*mut dyn FnMut())`
20250106    [X] layout vertices & indices use `Vec` instead of `HashMap`, because i think it might be slow as fuck when the data becomes too large
20250107    [X] callback using `Box<dyn FnMut(&mut T)>` & signal using `Rc<RefCell>`
20250108    [X] minimum redraw
20250112    [X] texture, bind group
20250112    [X] cpu side transform
20250116    [X] compile time texture collection
20250118    [X] gpu side transform via uniform buffer
20250118    [X] fix texture for each id
20250119    [X] signal injection & handling on each shape
20250125    [X] layouting (start from each widget) & later just scaling on storage
20250126    [X] fixed hover detection to the topmost object
20250128    [X] fixed layouting algorithm
20250216    [X] improve render performance (CPU side) -> use storage buffer
????????    [X] improve render performance (CPU side) -> reduce draw calls
????????    [ ] sdf -> position (layouting) based on shape's centerpoint
????????    [ ] better hover detection, currently using iterator
????????    [ ] dynamic (runtime) widget insertion
????????    [ ] render text, ttf parsing
