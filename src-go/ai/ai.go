package ai

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type AIService struct{}

func New() *AIService {
	return &AIService{}
}

// GenerateTests generates pm.test() scripts and YAML assertions for an API response.
func (s *AIService) GenerateTests(ctx context.Context, model, provider, apiKey, baseUrl, method, url, responseBody string, status int, headers map[string]string) (string, string, error) {
	prompt := fmt.Sprintf(`Analyze this API response and generate Postman-style pm.test() scripts and Parallax YAML assertions.

Request: %s %s
Response Status: %d
Response Body: %s

Generate at least 5 meaningful tests including status check, JSON structure validation, and value checks.
Return JSON with two fields: "js" (the pm.test code) and "yaml" (the parallax assertions).`, method, url, status, truncate(responseBody, 2000))

	return s.callProvider(ctx, provider, apiKey, baseUrl, model, prompt)
}

// RepairRequest analyzes a failed request and suggests fixes.
func (s *AIService) RepairRequest(ctx context.Context, model, provider, apiKey, baseUrl string,
	method, url string, reqHeaders map[string]string, reqBody string,
	status int, respBody string, envKeys []string) (string, error) {

	envList := ""
	for _, k := range envKeys {
		envList += "- " + k + "\n"
	}

	prompt := fmt.Sprintf(`You are an API debugging expert. A request failed. Analyze the problem and suggest fixes.

Request:
  Method: %s
  URL: %s
  Body: %s

Response:
  Status: %d
  Body: %s

Available environment variables (keys only):
%s

Return a JSON object with:
- "diagnosis": short human-readable explanation of the problem
- "fixes": array of { "type": "header|body|url|auth|env", "description": "...", "patch": {} }
- "priority": "high|medium|low"`, method, url, truncate(reqBody, 500), status, truncate(respBody, 1000), envList)

	js, _, err := s.callProvider(ctx, provider, apiKey, baseUrl, model, prompt)
	return js, err
}

// GenerateScript generates a pre-request or test script from a natural language prompt.
func (s *AIService) GenerateScript(ctx context.Context, model, provider, apiKey, baseUrl string,
	scriptType, userPrompt, method, url string) (string, error) {

	prompt := fmt.Sprintf(`Generate a Postman/Parallax %s script for this API request.

Request: %s %s
User request: %s

Return a JSON object with a single "script" field containing the JavaScript code.
Use the pm.* API. Include comments explaining each step.`, scriptType, method, url, userPrompt)

	js, _, err := s.callProvider(ctx, provider, apiKey, baseUrl, model, prompt)
	return js, err
}

// CreateCollection generates a full collection from a natural language description.
func (s *AIService) CreateCollection(ctx context.Context, model, provider, apiKey, baseUrl, description string) (string, error) {
	prompt := fmt.Sprintf(`You are an API collection designer. Create a Parallax collection YAML from this description.

Description: %s

Return a JSON object with a single "yaml" field containing a valid Parallax collection YAML with:
- Realistic folder structure
- Placeholder URLs using {{base_url}}
- Pre-request scripts that chain auth tokens
- Test assertions for each request
- An "environment" section listing all required variables`, description)

	js, _, err := s.callProvider(ctx, provider, apiKey, baseUrl, model, prompt)
	return js, err
}

// ─── Internal routing ────────────────────────────────────────────────────────

func (s *AIService) callProvider(ctx context.Context, provider, apiKey, baseUrl, model, prompt string) (string, string, error) {
	var result struct {
		JS     string `json:"js"`
		YAML   string `json:"yaml"`
		Script string `json:"script"`
		XYAML  string `json:"yaml"`
		Diagnosis string `json:"diagnosis"`
	}

	var err error
	switch provider {
	case "ollama":
		err = s.callOllama(ctx, baseUrl, model, prompt, &result)
	case "openai":
		err = s.callOpenAI(ctx, apiKey, "https://api.openai.com", model, prompt, &result)
	case "anthropic":
		err = s.callAnthropic(ctx, apiKey, model, prompt, &result)
	case "gemini":
		err = s.callGemini(ctx, apiKey, model, prompt, &result)
	case "custom":
		err = s.callOpenAI(ctx, apiKey, baseUrl, model, prompt, &result)
	default:
		return "", "", fmt.Errorf("provider %q not supported", provider)
	}

	if err != nil {
		return "", "", err
	}

	// Handle script-only responses (repair, generate-script, create-collection)
	out := result.JS
	if out == "" {
		out = result.Script
	}
	if out == "" {
		out = result.Diagnosis
	}
	return out, result.YAML, nil
}

// ─── Provider implementations ─────────────────────────────────────────────────

func (s *AIService) callOllama(ctx context.Context, baseUrl, model, prompt string, target interface{}) error {
	if baseUrl == "" {
		baseUrl = "http://localhost:11434"
	}
	url := baseUrl + "/api/generate"
	payload := map[string]interface{}{
		"model":  model,
		"prompt": prompt,
		"stream": false,
		"format": "json",
	}
	data, _ := json.Marshal(payload)
	req, _ := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 120 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Errorf("ollama request failed: %w", err)
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	var ollamaResp struct {
		Response string `json:"response"`
		Error    string `json:"error"`
	}
	if err := json.Unmarshal(body, &ollamaResp); err != nil {
		return fmt.Errorf("ollama response parse error: %w", err)
	}
	if ollamaResp.Error != "" {
		return fmt.Errorf("ollama error: %s", ollamaResp.Error)
	}

	if err := json.Unmarshal([]byte(ollamaResp.Response), target); err != nil {
		// Try wrapping in a js field as fallback
		wrapped := fmt.Sprintf(`{"js": %q}`, ollamaResp.Response)
		return json.Unmarshal([]byte(wrapped), target)
	}
	return nil
}

func (s *AIService) callOpenAI(ctx context.Context, apiKey, baseUrl, model, prompt string, target interface{}) error {
	if baseUrl == "" {
		baseUrl = "https://api.openai.com"
	}
	url := baseUrl + "/v1/chat/completions"
	payload := map[string]interface{}{
		"model": model,
		"messages": []map[string]string{
			{"role": "system", "content": "You are a helpful API testing assistant. Always respond with valid JSON."},
			{"role": "user", "content": prompt},
		},
		"response_format": map[string]string{"type": "json_object"},
		"temperature":     0.3,
	}

	data, _ := json.Marshal(payload)
	req, _ := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")
	if apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+apiKey)
	}

	client := &http.Client{Timeout: 60 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Errorf("openai request failed: %w", err)
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	var aiResp struct {
		Error   *struct{ Message string } `json:"error"`
		Choices []struct {
			Message struct{ Content string } `json:"message"`
		} `json:"choices"`
	}
	if err := json.Unmarshal(body, &aiResp); err != nil {
		return fmt.Errorf("openai response parse error: %w", err)
	}
	if aiResp.Error != nil {
		return fmt.Errorf("openai error: %s", aiResp.Error.Message)
	}
	if len(aiResp.Choices) == 0 {
		return fmt.Errorf("no response from OpenAI")
	}

	return json.Unmarshal([]byte(aiResp.Choices[0].Message.Content), target)
}

func (s *AIService) callAnthropic(ctx context.Context, apiKey, model, prompt string, target interface{}) error {
	if model == "" {
		model = "claude-3-5-sonnet-20241022"
	}
	url := "https://api.anthropic.com/v1/messages"
	payload := map[string]interface{}{
		"model":      model,
		"max_tokens": 4096,
		"system":     "You are a helpful API testing assistant. Always respond with valid JSON only, no markdown fences.",
		"messages": []map[string]string{
			{"role": "user", "content": prompt},
		},
	}

	data, _ := json.Marshal(payload)
	req, _ := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("x-api-key", apiKey)
	req.Header.Set("anthropic-version", "2023-06-01")

	client := &http.Client{Timeout: 60 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Errorf("anthropic request failed: %w", err)
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	var anthropicResp struct {
		Error *struct {
			Type    string `json:"type"`
			Message string `json:"message"`
		} `json:"error"`
		Content []struct {
			Type string `json:"type"`
			Text string `json:"text"`
		} `json:"content"`
	}
	if err := json.Unmarshal(body, &anthropicResp); err != nil {
		return fmt.Errorf("anthropic response parse error: %w", err)
	}
	if anthropicResp.Error != nil {
		return fmt.Errorf("anthropic error: %s", anthropicResp.Error.Message)
	}
	if len(anthropicResp.Content) == 0 {
		return fmt.Errorf("no response from Anthropic")
	}

	text := anthropicResp.Content[0].Text
	if err := json.Unmarshal([]byte(text), target); err != nil {
		wrapped := fmt.Sprintf(`{"js": %q}`, text)
		return json.Unmarshal([]byte(wrapped), target)
	}
	return nil
}

func (s *AIService) callGemini(ctx context.Context, apiKey, model, prompt string, target interface{}) error {
	if model == "" {
		model = "gemini-2.0-flash"
	}
	url := fmt.Sprintf("https://generativelanguage.googleapis.com/v1beta/models/%s:generateContent?key=%s", model, apiKey)

	payload := map[string]interface{}{
		"contents": []map[string]interface{}{
			{
				"parts": []map[string]string{
					{"text": "You are a helpful API testing assistant. Always respond with valid JSON only, no markdown fences.\n\n" + prompt},
				},
			},
		},
		"generationConfig": map[string]interface{}{
			"temperature":     0.3,
			"responseMimeType": "application/json",
		},
	}

	data, _ := json.Marshal(payload)
	req, _ := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 60 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Errorf("gemini request failed: %w", err)
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	var geminiResp struct {
		Error *struct {
			Message string `json:"message"`
		} `json:"error"`
		Candidates []struct {
			Content struct {
				Parts []struct {
					Text string `json:"text"`
				} `json:"parts"`
			} `json:"content"`
		} `json:"candidates"`
	}
	if err := json.Unmarshal(body, &geminiResp); err != nil {
		return fmt.Errorf("gemini response parse error: %w", err)
	}
	if geminiResp.Error != nil {
		return fmt.Errorf("gemini error: %s", geminiResp.Error.Message)
	}
	if len(geminiResp.Candidates) == 0 || len(geminiResp.Candidates[0].Content.Parts) == 0 {
		return fmt.Errorf("no response from Gemini")
	}

	text := geminiResp.Candidates[0].Content.Parts[0].Text
	if err := json.Unmarshal([]byte(text), target); err != nil {
		wrapped := fmt.Sprintf(`{"js": %q}`, text)
		return json.Unmarshal([]byte(wrapped), target)
	}
	return nil
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

func truncate(s string, n int) string {
	if len(s) <= n {
		return s
	}
	return s[:n] + "...[truncated]"
}
