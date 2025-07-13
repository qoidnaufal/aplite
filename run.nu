def main [name] {
    print $"cargo run --example ($name) --release"
    ^cargo run --example $name --release
}

def "main f" [name feature] {
    print $"cargo run --example ($name) --features ($feature) --release"
    ^cargo run --example $name --features $feature --release
}

def "main git" [message] {
    ^git add .
    ^git commit -m $message
    ^git push origin main
}

def "main log" [head] {
    ^git log --pretty=%h»¦«%aN»¦«%s»¦«%aD
    | lines
    | split column "»¦«" sha1 committer desc merged_at
    | first $head
}
