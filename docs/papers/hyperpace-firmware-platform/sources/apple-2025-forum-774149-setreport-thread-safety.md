URL: https://developer.apple.com/forums/thread/774149
Retrieved: 2026-09-15
Source type: Apple Developer Forums thread with Apple DTS engineer answer (Kevin Elliott, CoreOS/Hardware), February 2025.
Retrieval method: WebFetch (extraction model); quoted blocks were returned as block quotes.
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Title: "IOHIDDeviceSetReport thread safety"

Question:
> Can anyone advice on thread safety of IOHIDDeviceSetReport calls in IOKit framework? Any pointers to documentation covering the topic?

DTS engineer (accepted answer):
> My immediate question here is "what are you actually trying to"? In terms of "basic" thread safety, most IOKit APIs are "thread safe". The user client connection into the kernel and the in kernel driver itself both act as strong serialization points that protect the underlying hardware.
>
> However, most of these APIs are also bound to a particular run loop (see my take on runloops), which means they should really only be called on the same thread that as their run loop. Calling an API like that from a different thread may "work" (in the sense that nothing crashes) but that doesn't necessarily make it a good idea.
>
> Finally, by definition, an API like this is being used to manipulate the state of a physical dece and, presumably, your app would like to have a clear picture of that devices current or expected state... but that's not possible if you're manipulating that state from multiple states.

Follow-up from asker:
> "I have a legacy codebase which sets up a thread and registers IOHIDDevice callbacks with its runloop, but it also does IOHIDDeviceSetReport calls from multiple other threads. From what you wrote my understanding is that it 'may work', things will not crash, but in general it is a design to avoid and could cause weird behavior under some circumstances."

DTS engineer:
> Correct. More specifically, how well it works basically depends entirely on:
>
> * Exactly what you're doing with the accessory.
> * How the accessory itself behaves.
>
> In other words, most of the time the issue here isn't "the system failed" but is actually "you broke your accessory".
