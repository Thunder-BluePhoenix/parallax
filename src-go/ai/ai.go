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

func (s *AIService) GenerateTests(ctx context.Context, model, provider, apiKey, baseUrl, method, url, responseBody string, status int, headers map[string]string) (string, string, error) {
	prompt := fmt.Sprintf(`Analyze this API response and generate Postman-style pm.test() scripts and Parallax YAML assertions.

Request: %s %s
Response Status: %d
Response Body: %s

Generate at least 5 meaningful tests including status check, JSON structure validation, and value checks.
Return JSON with two fields: "js" (the pm.test code) and "yaml" (the parallax assertions).`, method, url, status, responseBody)

	var result struct {
		JS   string `json:"js"`
		YAML string `json:"yaml"`
	}

	switch provider {
	case "ollama":
		err := s.callOllama(ctx, baseUrl, model, prompt, &result)
		return result.JS, result.YAML, err
	case "openai":
		err := s.callOpenAI(ctx, apiKey, model, prompt, &result)
		return result.JS, result.YAML, err
	default:
		return "", "", fmt.Errorf("provider %s not yet implemented", provider)
	}
}

func (s *AIService) callOllama(ctx context.Context, baseUrl, model, prompt string, target interface{}) error {
	url := fmt.Sprintf("%s/api/generate", baseUrl)
	payload := map[string]interface{}{
		"model":  model,
		"prompt": prompt,
		"stream": false,
		"format": "json",
	}

	data, _ := json.Marshal(payload)
	req, _ := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 60 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	var ollamaResp struct {
		Response string `json:"response"`
	}
	if err := json.Unmarshal(body, &ollamaResp); err != nil {
		return err
	}

	return json.Unmarshal([]byte(ollamaResp.Response), target)
}

func (s *AIService) callOpenAI(ctx context.Context, apiKey, model, prompt string, target interface{}) error {
	url := "https://api.openai.com/v1/chat/completions"
	payload := map[string]interface{}{
		"model": model,
		"messages": []map[string]string{
			{"role": "user", "content": prompt},
		},
		"response_format": map[string]string{"type": "json_object"},
	}

	data, _ := json.Marshal(payload)
	req, _ := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(data))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+apiKey)

	client := &http.Client{Timeout: 60 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	var aiResp struct {
		Choices []struct {
			Message struct {
				Content string `json:"content"`
			} `json:"message"`
		} `json:"choices"`
	}
	if err := json.Unmarshal(body, &aiResp); err != nil {
		return err
	}

	if len(aiResp.Choices) == 0 {
		return fmt.Errorf("no response from OpenAI")
	}

	return json.Unmarshal([]byte(aiResp.Choices[0].Message.Content), target)
}
