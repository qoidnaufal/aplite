# TODO
20250104    [X] rewrite shape & layout, i need to preserve the shape's size etc<br>
20250104    [X] fix cursor to stick only to the currently clicked object, to avoid hover collision<br>
20250106    [X] function callback using `CallBack(*mut dyn FnMut())`<br>
20250106    [X] layout vertices & indices use `Vec` instead of `HashMap`, because i think it might be slow as fuck when the data becomes too large<br>
20250107    [X] callback using `Box<dyn FnMut(&mut T)>` & signal using `Rc<RefCell>`<br>
20250108    [X] minimum redraw<br>
20250112    [X] texture, bind group<br>
20250112    [X] cpu side transform<br>
20250116    [X] compile time texture collection<br>
20250118    [X] gpu side transform via uniform buffer<br>
20250118    [X] fix texture for each id<br>
20250119    [X] signal injection & handling on each shape<br>
20250125    [X] layouting (start from each widget) & later just scaling on storage<br>
20250126    [X] fixed hover detection to the topmost object<br>
20250128    [X] fixed layouting algorithm<br>
20250216    [X] improve render performance (CPU side) -> use storage buffer<br>
????????    [X] improve render performance (CPU side) -> reduce draw calls<br>
????????    [ ] sdf -> position (layouting) based on shape's centerpoint<br>
????????    [ ] better hover detection, currently using iterator<br>
????????    [ ] dynamic (runtime) widget insertion<br>
????????    [ ] render text, ttf parsing<br>
