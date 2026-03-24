# Bark Demo — VHS recording script
# Usage: brew install vhs && vhs demo.tape
# Generates: demo.gif

Output demo.gif
Set Shell "bash"
Set FontSize 14
Set Width 900
Set Height 550
Set Padding 20
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms

# 1. Check status
Type "bark status"
Enter
Sleep 2s

# 2. Test: safe command — instant allow
Type "bark test ls -la"
Enter
Sleep 2s

# 3. Test: medium risk — auto-allow with notification
Type "bark test npm install express"
Enter
Sleep 8s

# 4. Test: high risk — blocked!
Type "bark test rm -rf /"
Enter
Sleep 8s

# 5. Verbose mode — show evaluation flow
Type "bark test --verbose git push"
Enter
Sleep 8s

# 6. Stats dashboard
Type "bark stats"
Enter
Sleep 3s

# 7. Custom rules
Type "bark rules"
Enter
Sleep 2s

# 8. Version
Type "bark version"
Enter
Sleep 2s
