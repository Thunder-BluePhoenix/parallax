import toJsonSchema from 'to-json-schema';

// ── Pydantic (Python) ─────────────────────────────────────────────────────────

export function inferPydantic(data: any, className: string = 'Response'): string {
  const schema = inferJsonSchema(data);
  const classes: string[] = [];

  function pyType(details: any, name: string = ''): string {
    switch (details.type) {
      case 'string': return 'str';
      case 'number': return 'float';
      case 'integer': return 'int';
      case 'boolean': return 'bool';
      case 'null': return 'None';
      case 'array': {
        const inner = details.items ? pyType(details.items, name + 'Item') : 'Any';
        return `list[${inner}]`;
      }
      case 'object': return generateClass(details, toPascal(name) || 'SubModel');
      default: return 'Any';
    }
  }

  function generateClass(obj: any, name: string): string {
    if (obj.type !== 'object' || !obj.properties) return 'Any';
    const safeName = toPascal(name);
    const fields = Object.entries(obj.properties as Record<string, any>)
      .map(([k, v]) => `    ${k}: ${pyType(v, k)}`)
      .join('\n');
    classes.push(`class ${safeName}(BaseModel):\n${fields || '    pass'}`);
    return safeName;
  }

  generateClass(schema, className);
  return `from pydantic import BaseModel\nfrom typing import Any\n\n${classes.reverse().join('\n\n')}\n`;
}

// ── Rust struct ───────────────────────────────────────────────────────────────

export function inferRustStruct(data: any, structName: string = 'Response'): string {
  const schema = inferJsonSchema(data);
  const structs: string[] = [];

  function rustType(details: any, name: string = ''): string {
    switch (details.type) {
      case 'string': return 'String';
      case 'number': return 'f64';
      case 'integer': return 'i64';
      case 'boolean': return 'bool';
      case 'null': return '()';
      case 'array': {
        const inner = details.items ? rustType(details.items, name + 'Item') : 'serde_json::Value';
        return `Vec<${inner}>`;
      }
      case 'object': return generateStruct(details, toPascal(name) || 'SubStruct');
      default: return 'serde_json::Value';
    }
  }

  function generateStruct(obj: any, name: string): string {
    if (obj.type !== 'object' || !obj.properties) return 'serde_json::Value';
    const safeName = toPascal(name);
    const fields = Object.entries(obj.properties as Record<string, any>)
      .map(([k, v]) => `    pub ${toSnake(k)}: ${rustType(v, k)},`)
      .join('\n');
    structs.push(`#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ${safeName} {\n${fields}\n}`);
    return safeName;
  }

  generateStruct(schema, structName);
  return `use serde::{Deserialize, Serialize};\n\n${structs.reverse().join('\n\n')}\n`;
}

// ── Go struct ─────────────────────────────────────────────────────────────────

export function inferGoStruct(data: any, structName: string = 'Response'): string {
  const schema = inferJsonSchema(data);
  const structs: string[] = [];

  function goType(details: any, name: string = ''): string {
    switch (details.type) {
      case 'string': return 'string';
      case 'number': return 'float64';
      case 'integer': return 'int64';
      case 'boolean': return 'bool';
      case 'null': return 'interface{}';
      case 'array': {
        const inner = details.items ? goType(details.items, name + 'Item') : 'interface{}';
        return `[]${inner}`;
      }
      case 'object': return generateStruct(details, toPascal(name) || 'SubStruct');
      default: return 'interface{}';
    }
  }

  function generateStruct(obj: any, name: string): string {
    if (obj.type !== 'object' || !obj.properties) return 'interface{}';
    const safeName = toPascal(name);
    const fields = Object.entries(obj.properties as Record<string, any>)
      .map(([k, v]) => `\t${toPascal(k)} ${goType(v, k)} \`json:"${k}"\``)
      .join('\n');
    structs.push(`type ${safeName} struct {\n${fields}\n}`);
    return safeName;
  }

  generateStruct(schema, structName);
  return `package main\n\n${structs.reverse().join('\n\n')}\n`;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function toPascal(s: string): string {
  return s.replace(/(^|[-_\s])(\w)/g, (_, __, c) => c.toUpperCase());
}

function toSnake(s: string): string {
  return s.replace(/([A-Z])/g, '_$1').replace(/^_/, '').replace(/-/g, '_').toLowerCase();
}

export function inferJsonSchema(data: any): any {
  if (data === null || data === undefined) return { type: 'null' };
  
  return toJsonSchema(data, {
    required: false,
    objects: {
      additionalProperties: false
    },
    arrays: {
      mode: 'first' // 'first' is usually better for API docs than 'all' which can get messy
    }
  });
}

export function inferTypeScript(data: any, interfaceName: string = 'Root'): string {
  const schema = inferJsonSchema(data);
  const lines: string[] = [];
  const processedObjects = new Map<string, string>();

  function generateInterface(obj: any, name: string): string {
    if (obj.type !== 'object' || !obj.properties) {
      return getTsType(obj);
    }

    const cacheKey = JSON.stringify(obj.properties);
    if (processedObjects.has(cacheKey)) {
      return processedObjects.get(cacheKey)!;
    }

    const safeName = name.charAt(0).toUpperCase() + name.slice(1);
    processedObjects.set(cacheKey, safeName);

    let interfaceStr = `export interface ${safeName} {\n`;
    for (const [prop, details] of Object.entries(obj.properties as any)) {
      const typeStr = getTsType(details, prop);
      interfaceStr += `  ${prop}: ${typeStr};\n`;
    }
    interfaceStr += `}\n`;
    
    lines.push(interfaceStr);
    return safeName;
  }

  function getTsType(details: any, name: string = ''): string {
    switch (details.type) {
      case 'string': return 'string';
      case 'number':
      case 'integer': return 'number';
      case 'boolean': return 'boolean';
      case 'null': return 'null';
      case 'array':
        const itemType = details.items ? getTsType(details.items, name + 'Item') : 'any';
        return `${itemType}[]`;
      case 'object':
        return generateInterface(details, name || 'SubObject');
      default:
        return 'any';
    }
  }

  const rootType = getTsType(schema, interfaceName);
  
  // If the root was an object, the interface is already in 'lines'
  // If the root was a primitive or array, we add a type alias
  if (schema.type !== 'object') {
    return `export type ${interfaceName} = ${rootType};\n\n` + lines.join('\n');
  }

  return lines.reverse().join('\n');
}
