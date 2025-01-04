Output demo.gif

Set Shell "bash"
Set FontSize 32
Set Width 1600
Set Height 900
Set PlaybackSpeed 0.7
Set Theme "Catppuccin Mocha"

Type "# Let's try ghif - GitHub Issue Fetcher!"
Enter
Sleep 1s

Type "# First, let's fetch all issues from a repository"
Enter
Sleep 1s

Type "ghif -r scalar/scalar"
Enter
Sleep 5s
Ctrl+c

Type "# Now let's check the downloaded issues"
Enter
Sleep 1s

Type "ls issues | head -n 5"
Enter
Sleep 1s

Type "# Let's look at one of the issues"
Enter
Sleep 2s

Type "head -n 10 issues/$(ls issues | grep issue | head -n 1)"
Enter
Sleep 3s

Type "# We can also fetch specific issues"
Enter
Sleep 1s

Type "ghif -r scalar/scalar -i 1,2 --format xml"
Enter
Sleep 5s
Ctrl+c

Type "# That's it! Easy to use, right? 🚀"
Enter
Sleep 1s