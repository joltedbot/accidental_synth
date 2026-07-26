# Apple Gatekeeper Security Workaround

Accidental Synthesizer is a free, single developer build app. As such, I haven't paid Apple for Developer ID to have it signed. The first time you open it macOS Gatekeeper freak out and try and stop you using it. 
It will tell you that it is from an "unidentified developer" or that is broken, or that it will end the world if you use it, or something like that.

Either way it wont let you load it right away. Thanks, Apple!

I could tell you that I believe it is safe, and I have tried to make it so, but there is no reason you should believe me so there is not much 
point. The code is [here](https://gitlab.com/joltedbot-public/accidental-synth) though if you want to give it a read. 

If you do still want to run it though and want to bypass Apple trying to make me pay them to give you free software then this should do it.

Be careful disabling security for this or any application on you computer. Do your homework and don't blame me if you manage to cause your self 
some problems.

If you open the App for the first time and it throws a bunch of warnings, you can go to `Settings` -> `Privacy & Security` and near the bottom 
click the `Open Anyway` button.

If this doesn't work, or you don't get that option, and instead it says the app is broken and just fails outright then you need to go nuclear.

### The bit you want is down here
From a Terminal, remove the quarantine flag before launching:

```
xattr -cr "/Applications/Accidental Synthesizer.app"
```

You should only need to do this once per version you install.
