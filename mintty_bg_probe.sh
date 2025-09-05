# Save this as mintty_bg_probe.sh and run: bash mintty_bg_probe.sh
# printf '\033]11;?\a'  # send OSC 11 query
# printf '\033]11;?\007'    # BEL terminator
printf '\033]11;?\033\\'  # ST terminator

# now read the response (with timeout so it doesn't hang forever)
# expects mintty to respond with something like: ESC ] 11;rgb:5c5c/3f3f/1515 BEL
IFS= read -r -t 1 reply
echo "Response: $reply" | od -An -tx1
