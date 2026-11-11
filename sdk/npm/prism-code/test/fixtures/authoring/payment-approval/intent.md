# What payment approval must always and never do

A payment above the limit never returns approved.
A payment approved by the person who requested it never returns approved.
A payment always returns approved when it is at most the limit and the approver is not the requester.
A request with a version other than 1 never returns approved.
A malformed request never returns approved.
