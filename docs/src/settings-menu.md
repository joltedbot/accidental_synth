# Settings Menu

## Audio Settings

| Setting | Description |
|---------|-------------|
| **Output Device** | The audio output device AccSyn will use. Changes take effect immediately. |
| **Left Channel** | Which output channel carries the left stereo signal. |
| **Right Channel** | Which output channel carries the right stereo signal. |
| **Sample Rate** | Audio sample rate in Hz. Higher rates give more high-frequency headroom at the cost of CPU. |
| **Buffer Size** | Audio buffer size in samples. Smaller buffers reduce latency; larger buffers reduce the risk of dropouts. |

To record AccSyn in your DAW either send it through audio loopback software or do what I do and assign it to a stereo pair of output channels on your audio interface and patch them back to a stereo pair of input channels. Then you can treat AccSyn like a hardware synth and route it through outboard effects, a mixer, or what ever else you do with your synths.

## MIDI Settings

| Setting | Description |
|---------|-------------|
| **Input Port** | The MIDI input device AccSyn listens to. AccSyn also creates a virtual port named **AccSyn MIDI Input** that is always available. MIDI devices can be changed at any time without restarting. |
| **Channel** | The MIDI channel AccSyn responds to. Defaults to Omni (all channels). Set to a specific channel (1–16) to ignore messages on other channels. The Input port and the Virtual Port share the same channel setting. |

To prevent needing to use MIDI loop back software to connect to AccSyn from you DAW you can simply point it at the AccSyn virtual port. This port is available no matter what you have chosen for the Input port. The two can also be used simultaneously if you wanted to say send notes to one and CC or clock to the other. 

## Synth Options

### _Polarity_
This option swaps the polarity of the waveform the synth outputs. It is a simple polarity inversion at the very end of the signal chain. The change will not be audible unless you are getting some kind of interference with another signal's phase/polarity.

If you find that you are getting some kind of cancelation or destructive interference, say with a bass patch and your kick, you can try swapping the polarity. It shouldn't be needed most of the time but if you aren't using a DAW where you can do the polarity swap this might come in handy in certain circumstances.


## User Patches

In this section you are able to save the current state of the Synth And Effects controls as a user patch. You can also delete previously saved patches. See the [Patch Format](./patch-format.md) section for full details on patches.

### _Save a Patch_

This allows you to save the current state of the Synth and Effects tabs. The settings menu state is not saved in patches. Files will be saved using the name provided. Patch names can be a maximum of 24 characters long. The patch name is used for the file name so some character restrictions apply. Names can include upper and lower case letters (including some accented characters), spaces, numbers, as well as most special characters.

### _Delete a Patch_

Choose the patch to be deleted and press delete. `WARNING!: This CAN NOT be undone.`
