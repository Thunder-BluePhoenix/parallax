package models

type Collection struct {
	Name      string               `yaml:"name"`
	Variables map[string]string    `yaml:"variables"`
	Folders   []Folder             `yaml:"folders"`
	Requests  []CollectionRequest  `yaml:"requests"`
}

type Folder struct {
	Name     string               `yaml:"name"`
	Requests []CollectionRequest  `yaml:"requests"`
}

type CollectionRequest struct {
	ID      string            `yaml:"id"`
	Name    string            `yaml:"name"`
	Method  string            `yaml:"method"`
	URL     string            `yaml:"url"`
	Headers map[string]string `yaml:"headers"`
	Params  map[string]string `yaml:"params"`
	Body    *RequestBody      `yaml:"body"`
	Auth    *Auth             `yaml:"auth"`
	Scripts *RequestScripts   `yaml:"scripts"`
}

type RequestScripts struct {
	PreRequest string `yaml:"preRequest"`
	Tests      string `yaml:"tests"`
}

type RequestBody struct {
	Type    string `yaml:"type"`
	Content string `yaml:"content"`
}

type Auth struct {
	Type     string `yaml:"auth_type"`
	Token    string `yaml:"token"`
	Username string `yaml:"username"`
	Password string `yaml:"password"`
}
