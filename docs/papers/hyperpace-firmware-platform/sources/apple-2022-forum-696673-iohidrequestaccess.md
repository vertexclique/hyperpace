URL: https://developer.apple.com/forums/thread/696673
Retrieved: 2026-09-15
Source type: Apple Developer Forums thread (Dec 2021 to Jan 2022) with an Apple DTS reply (Quinn "The Eskimo!").
Retrieval method: WebFetch (extraction model); quotes were returned in quotation marks.
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Title: "Question about IOHIDRequestAccess"

Question (mise):
> "I use a program that needs Input monitoring to work, I am using the following to check:
> `bool checkPermission = IOHIDRequestAccess(kIOHIDRequestTypeListenEvent);`
> This works in principle but does not work in real time; for example, if I uncheck my application's box in Input Monitoring in Security and Privacy the method keeps throwing me 1 as if I had permission, unless I restart the application instance, then it will show 0."

Accepted answer (robnotyou, community):
> "The changes are not instant, but take effect only when the app is quit and relaunched.
> If the app is running when the change is made in System Preferences, I see a dialog confirming this (with an option to "Quit & Reopen")."

DTS engineer (Quinn):
> "The subsystem that tracks these permissions is known as TCC (short for transparency, consent, control). It is quite complex and it's certainly had its fair share of bugs over the years )-:"

> "The one critical step that you can take to avoid TCC problems is to make sure that your code is signed with a stable DR, such that build N+1 of your code has the same DR as build N. Any standard signing identity issued by Apple will give you this."
