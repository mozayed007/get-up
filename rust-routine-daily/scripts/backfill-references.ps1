$ErrorActionPreference = "Stop"

function Get-HelloAlgoReference {
    param([string[]]$Tags)
    foreach ($tag in $Tags) {
        $result = switch ($tag) {
            "array"               { @{url = "https://www.hello-algo.com/en/chapter_array_and_linkedlist/"; display = "Array & Linked List" }; break }
            "string"              { @{url = "https://www.hello-algo.com/en/chapter_array_and_linkedlist/"; display = "Array & Linked List" }; break }
            "matrix"              { @{url = "https://www.hello-algo.com/en/chapter_array_and_linkedlist/"; display = "Array & Linked List" }; break }
            "hash-table"          { @{url = "https://www.hello-algo.com/en/chapter_hashing/"; display = "Hash Table" }; break }
            "linked-list"         { @{url = "https://www.hello-algo.com/en/chapter_array_and_linkedlist/linked_list/"; display = "Linked List" }; break }
            "binary-search"       { @{url = "https://www.hello-algo.com/en/chapter_searching/binary_search/"; display = "Binary Search" }; break }
            "divide-and-conquer"  { @{url = "https://www.hello-algo.com/en/chapter_divide_and_conquer/"; display = "Divide & Conquer" }; break }
            "dynamic-programming" { @{url = "https://www.hello-algo.com/en/chapter_dynamic_programming/intro_to_dynamic_programming/"; display = "Dynamic Programming" }; break }
            "backtracking"        { @{url = "https://www.hello-algo.com/en/chapter_backtracking/"; display = "Backtracking" }; break }
            "greedy"              { @{url = "https://www.hello-algo.com/en/chapter_greedy/"; display = "Greedy" }; break }
            "sorting"             { @{url = "https://www.hello-algo.com/en/chapter_sorting/"; display = "Sorting" }; break }
            "counting-sort"       { @{url = "https://www.hello-algo.com/en/chapter_sorting/"; display = "Sorting" }; break }
            "bucket-sort"         { @{url = "https://www.hello-algo.com/en/chapter_sorting/"; display = "Sorting" }; break }
            "radix-sort"          { @{url = "https://www.hello-algo.com/en/chapter_sorting/"; display = "Sorting" }; break }
            "heap"                { @{url = "https://www.hello-algo.com/en/chapter_heap/"; display = "Heap" }; break }
            "priority-queue"      { @{url = "https://www.hello-algo.com/en/chapter_heap/"; display = "Heap" }; break }
            "tree"                { @{url = "https://www.hello-algo.com/en/chapter_tree/binary_tree/"; display = "Binary Tree" }; break }
            "binary-tree"         { @{url = "https://www.hello-algo.com/en/chapter_tree/binary_tree/"; display = "Binary Tree" }; break }
            "binary-search-tree"  { @{url = "https://www.hello-algo.com/en/chapter_tree/binary_tree/"; display = "Binary Tree" }; break }
            "depth-first-search"  { @{url = "https://www.hello-algo.com/en/chapter_tree/binary_tree_traversal/"; display = "Binary Tree Traversal" }; break }
            "breadth-first-search" { @{url = "https://www.hello-algo.com/en/chapter_graph/graph_traversal/"; display = "Graph Traversal" }; break }
            "graph"               { @{url = "https://www.hello-algo.com/en/chapter_graph/"; display = "Graph" }; break }
            "union-find"          { @{url = "https://www.hello-algo.com/en/chapter_graph/"; display = "Graph" }; break }
            "stack"               { @{url = "https://www.hello-algo.com/en/chapter_stack_and_queue/stack/"; display = "Stack" }; break }
            "monotonic-stack"     { @{url = "https://www.hello-algo.com/en/chapter_stack_and_queue/stack/"; display = "Stack" }; break }
            "queue"               { @{url = "https://www.hello-algo.com/en/chapter_stack_and_queue/queue/"; display = "Queue" }; break }
            "monotonic-queue"     { @{url = "https://www.hello-algo.com/en/chapter_stack_and_queue/queue/"; display = "Queue" }; break }
            "recursion"           { @{url = "https://www.hello-algo.com/en/chapter_computational_complexity/iteration_and_recursion/"; display = "Recursion" }; break }
            "trie"                { @{url = "https://www.hello-algo.com/en/chapter_tree/binary_tree/"; display = "Trie" }; break }
            "two-pointers"        { @{url = "https://www.hello-algo.com/en/chapter_array_and_linkedlist/"; display = "Two Pointers" }; break }
            "prefix-sum"          { @{url = "https://www.hello-algo.com/en/chapter_searching/"; display = "Prefix Sum" }; break }
            "sliding-window"      { @{url = "https://www.hello-algo.com/en/chapter_stack_and_queue/"; display = "Sliding Window" }; break }
            "bit-manipulation"    { @{url = "https://www.hello-algo.com/en/chapter_data_structure/number_encoding/"; display = "Bit Manipulation" }; break }
            "design"              { @{url = "https://www.hello-algo.com/en/chapter_array_and_linkedlist/list/"; display = "List & Iterator" }; break }
            "iterator"            { @{url = "https://www.hello-algo.com/en/chapter_array_and_linkedlist/list/"; display = "List & Iterator" }; break }
        }
        if ($result) { return $result }
    }
    return $null
}

$comments = gh api repos/mozayed007/get-up/issues/1/comments --jq '.' | ConvertFrom-Json
$total = $comments.Count
$processed = 0
$edited = 0
$skipped = 0
$failed = 0
$slugCache = @{}

foreach ($comment in $comments) {
    $processed++
    $body = $comment.body
    Write-Host "[$processed/$total] Comment $($comment.id) ($($comment.created_at))..."

    if ($body -match '\n📖 ') {
        Write-Host "  Already has reference lines, skipping."
        $skipped++
        continue
    }

    $urlMatches = [regex]::Matches($body, 'https://leetcode\.com/problems/([^/\s?#]+)')
    if ($urlMatches.Count -eq 0) {
        Write-Host "  No LeetCode URLs found, skipping."
        $skipped++
        continue
    }

    $slugData = @()
    $seen = @{}
    foreach ($match in $urlMatches) {
        $slug = $match.Groups[1].Value
        if (-not $seen.ContainsKey($slug)) {
            $seen[$slug] = $true
            $slugData += @{ slug = $slug; index = $match.Index; length = $match.Length }
        }
    }

    $slugData = $slugData | Sort-Object -Property index -Descending

    $newBody = $body
    $insertedCount = 0
    $insertErrors = 0

    foreach ($sd in $slugData) {
        $slug = $sd.slug

        if ($slugCache.ContainsKey($slug)) {
            $ref = $slugCache[$slug]
        } else {
            Write-Host "  Fetching tags for $slug..."
            try {
                $query = @{
                    query = "query questionTopicTags(`$titleSlug: String!) { question(titleSlug: `$titleSlug) { topicTags { slug } } }"
                    variables = @{ titleSlug = $slug }
                } | ConvertTo-Json -Depth 10

                $response = Invoke-RestMethod -Uri "https://leetcode.com/graphql/" -Method Post `
                    -Headers @{ "User-Agent" = "LeetCodeDaily/0.2.0" } `
                    -Body $query -ContentType "application/json" -ErrorAction Stop
                $tags = $response.data.question.topicTags | ForEach-Object { $_.slug }
                $ref = Get-HelloAlgoReference -Tags $tags
                $slugCache[$slug] = $ref
                Write-Host "    Tags: $($tags -join ', ') -> $($ref.display)"
            } catch {
                Write-Host "    Failed: $_"
                $ref = $null
                $slugCache[$slug] = $null
            }
        }

        if (-not $ref) { continue }

        $referenceLine = "📖 [$($ref.display)]($($ref.url))"

        $urlInBody = $newBody.IndexOf("https://leetcode.com/problems/$slug")
        if ($urlInBody -lt 0) { continue }

        $lineStart = $newBody.LastIndexOf("`n", $urlInBody)
        if ($lineStart -lt 0) { $lineStart = 0 } else { $lineStart++ }

        $lineEnd = $newBody.IndexOf("`n", $urlInBody)
        if ($lineEnd -lt 0) { $lineEnd = $newBody.Length }

        $urlLine = $newBody.Substring($lineStart, $lineEnd - $lineStart)

        $newBody = $newBody.Substring(0, $lineStart) + $referenceLine + "`n" + $newBody.Substring($lineStart)
        $insertedCount++
    }

    if ($insertedCount -gt 0) {
        Write-Host "  Updating comment ($insertedCount references)..."
        $tmpFile = [System.IO.Path]::GetTempFileName()
        try {
            @{ body = $newBody } | ConvertTo-Json -Depth 3 -Compress | Set-Content -Path $tmpFile -Encoding UTF8
            $result = gh api "repos/mozayed007/get-up/issues/comments/$($comment.id)" --method PATCH --input "$tmpFile" 2>&1
            if ($LASTEXITCODE -eq 0) {
                $edited++
                Write-Host "  Done."
            } else {
                Write-Host "  FAILED: $result"
                $failed++
            }
        } catch {
            Write-Host "  FAILED: $_"
            $failed++
        } finally {
            Remove-Item -Path $tmpFile -Force -ErrorAction SilentlyContinue
        }
    } else {
        Write-Host "  No references to insert (no matching tags)."
        $skipped++
    }

    Start-Sleep -Milliseconds 500
}

Write-Host "`nDone. Processed: $processed, Edited: $edited, Skipped: $skipped, Failed: $failed"