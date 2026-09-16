# decs

Decision logs from autonomous loop runs. One file per plan: <PLAN_NAME>_DECS.md.

When vertexia-loop runs a plan to completion without stopping, it records every
decision it takes here, newest first, so the operator reviews the whole trail
after the run. The loop never decides silently: if it is not asking (because a
loop run is pre-authorized to finish), it is logging. See templates/decisions.md.
