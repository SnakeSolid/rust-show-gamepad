# Show Gamepad

Simple visualizer for gamepad buttons. Can be used for streaming.

![Screenshots](images/image.png "Screenshots")

## Usage

Press `F1` to start key binding. Screen will show sprite name and active
keys. Hold keys corresponding to current sprite and press `F1` again to
select next sprite. After binding last sprite this combination will be
saved to preferences.

If application show pressed buttons (sticks or shoulders) which actually
does not. Press `F2` to save actual positions as default axes value.

Application support all joysticks and keyboard. Keyboard will work only in
active window and can be used to test configuration changes.

Application support settings for several joysticks at the same time. In this
case all joystick will be show in the same window simultaneously.

## Options

Default sprites for buttons are placed to `sprites` directory. Sprites can be
changed in any graphic editor, but dimensions of all sprites must be the same.
Window size will be changed depending on image used as `background` in
configuration.

Configuration description:

```yaml
background: "sprites/controller.png" # background image

sprites: # button sprites for button visualization
    - group: 1 # button group, usually corresponds to hand.
               # Only one sprite from group can be shown at once.
      name: "Up" # Sprite name, will be show during button binding
      path: "sprites/controller-up.png" # path to sprite image
      default: false # use this sprite as default image for group.
                     # Sprite will be shown when other sprites
                     # not match current input state
    - { group: 1, name: "Up Right", path: "sprites/controller-up-right.png" }
```

## License
[license]: #license

Source code is primarily distributed under the terms of the MIT license. See LICENSE for details.
