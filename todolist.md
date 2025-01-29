# TODO
20240104    [X] rewrite shape & layout, i need to preserve the shape's size etc
20240104    [X] fix cursor to stick only to the currently clicked object, to avoid hover collision
20240106    [X] function callback using `CallBack(*mut dyn FnMut())`
20240106    [X] layout vertices & indices use `Vec` instead of `HashMap`, because i think it might be slow as fuck when the data becomes too large
20240107    [X] callback using `Box<dyn FnMut(&mut T)>` & signal using `Rc<RefCell>`
20240108    [X] minimum redraw
20240112    [X] texture, bind group
20240112    [X] cpu side transform
20240116    [X] compile time texture collection
20240118    [X] gpu side transform via uniform buffer
20240118    [X] fix texture for each id
20240119    [X] signal injection & handling on each shape
20240125    [X] layouting (start from each widget) & later just scaling on storage
20240126    [X] fixed hover detection to the topmost object
20240128    [X] fixed layouting algorithm
????????    [ ] sdf -> position (layouting) based on shape's centerpoint
????????    [ ] improve render performance (CPU side)
????????    [ ] better hover detection, currently using iterator
????????    [ ] dynamic (runtime) widget insertion
????????    [ ] render text, ttf parsing
