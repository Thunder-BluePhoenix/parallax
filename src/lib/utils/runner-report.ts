export const runnerReportTemplate = `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Parallax — Collection Run Report</title>
    <style>
        :root {
            --bg: #0f1115;
            --surface: #1a1d23;
            --text: #e1e4e8;
            --text-muted: #8b949e;
            --accent: #7c6fef;
            --success: #3fb950;
            --error: #f85149;
            --border: #30363d;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: var(--bg);
            color: var(--text);
            margin: 0;
            padding: 40px;
        }
        .container { max-width: 1000px; margin: 0 auto; }
        header { margin-bottom: 40px; display: flex; justify-content: space-between; align-items: flex-end; border-bottom: 1px solid var(--border); padding-bottom: 20px; }
        h1 { margin: 0; font-size: 24px; font-weight: 800; color: var(--accent); }
        .timestamp { font-size: 14px; color: var(--text-muted); }
        
        .summary { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; margin-bottom: 40px; }
        .stat-card { background: var(--surface); padding: 20px; border-radius: 12px; border: 1px solid var(--border); }
        .stat-val { font-size: 32px; font-weight: 700; display: block; }
        .stat-label { font-size: 12px; text-transform: uppercase; color: var(--text-muted); letter-spacing: 0.05em; }
        
        .feed { display: flex; flex-direction: column; gap: 12px; }
        .feed-item { background: var(--surface); border: 1px solid var(--border); border-radius: 12px; overflow: hidden; }
        .feed-header { padding: 16px 20px; display: flex; align-items: center; gap: 16px; border-bottom: 1px solid var(--border); }
        .method { font-family: monospace; font-weight: 700; font-size: 12px; padding: 4px 8px; border-radius: 4px; background: rgba(255,255,255,0.1); }
        .name { flex: 1; font-weight: 600; }
        .status { font-family: monospace; font-weight: 700; }
        .status.pass { color: var(--success); }
        .status.fail { color: var(--error); }
        
        .tests { padding: 12px 20px; background: rgba(0,0,0,0.2); }
        .test { display: flex; align-items: center; gap: 10px; font-size: 14px; margin: 6px 0; }
        .icon { font-weight: 900; }
        .icon.pass { color: var(--success); }
        .icon.fail { color: var(--error); }
        .test-name { flex: 1; }
        .test-err { font-family: monospace; font-size: 12px; color: var(--error); opacity: 0.8; }
        
        .text-success { color: var(--success); }
        .text-error { color: var(--error); }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <div>
                <h1>PARALLAX RUN REPORT</h1>
                <div class="timestamp">Generated on {{timestamp}}</div>
            </div>
            <div style="text-align: right">
                <div style="font-weight: 600">{{collectionName}}</div>
                <div style="font-size: 12px; color: var(--text-muted)">{{folderName}}</div>
            </div>
        </header>

        <div class="summary">
            <div class="stat-card">
                <span class="stat-val text-success">{{stats.passed}}</span>
                <span class="stat-label">Tests Passed</span>
            </div>
            <div class="stat-card">
                <span class="stat-val text-error">{{stats.failed}}</span>
                <span class="stat-label">Tests Failed</span>
            </div>
            <div class="stat-card">
                <span class="stat-val">{{stats.timeMs}}ms</span>
                <span class="stat-label">Total Duration</span>
            </div>
            <div class="stat-card">
                <span class="stat-val">{{requests.length}}</span>
                <span class="stat-label">Total Requests</span>
            </div>
        </div>

        <div class="feed">
            {{#each requests}}
            <div class="feed-item">
                <div class="feed-header">
                    <span class="method">{{method}}</span>
                    <span class="name">{{reqName}}</span>
                    <span class="status {{#if error}}fail{{else}}pass{{/if}}">{{#if status}}{{status}}{{else}}ERR{{/if}}</span>
                    <span class="time" style="font-family: monospace; font-size: 12px; color: var(--text-muted)">{{timeMs}}ms</span>
                </div>
                {{#if tests.length}}
                <div class="tests">
                    {{#each tests}}
                    <div class="test">
                        <span class="icon {{#if passed}}pass{{else}}fail{{/if}}">{{#if passed}}✓{{else}}✗{{/if}}</span>
                        <span class="test-name">{{name}}</span>
                        {{#if error}}<span class="test-err">({{error}})</span>{{/if}}
                    </div>
                    {{/each}}
                </div>
                {{/if}}
            </div>
            {{/each}}
        </div>
    </div>
</body>
</html>
`;
