# Localization Guide for Bicep-Docs

This document provides guidelines for contributors who want to add or improve translations for the bicep-docs project.

## Overview

Bicep-docs supports internationalization (i18n) for both the CLI interface and generated documentation. The localization system uses YAML-based translation files and supports automatic fallback to English for missing translations.

## Supported Languages

Currently supported languages:
- **English (en)** - Default/fallback language
- **Spanish (es)** - Español
- **French (fr)** - Français  
- **German (de)** - Deutsch
- **Japanese (ja)** - 日本語
- **Chinese (zh)** - 中文

## Translation Files Structure

Translation files are located in `locales/` and follow the YAML format with nested objects using dot notation keys:

```
locales/
├── en.yml    # English (reference)
├── es.yml    # Spanish
├── fr.yml    # French
├── de.yml    # German
├── ja.yml    # Japanese
└── zh.yml    # Chinese
```

### Translation File Format

Each translation file is a YAML object with nested sections:

```yaml
cli:
  app_description: "Documentation generator for Azure Bicep files"
  verbose_help: "Set the verbosity level of output"
  language_help: "Set the language for CLI messages and generated documentation"
export:
  bicep_template: "Bicep Template"
  target_scope: "Target Scope"
  types: "Types"
  parameters: "Parameters"
common:
  yes: "Yes"
  no: "No"
error:
  file_not_found: "File not found"
  parse_error: "Parse error"
```

### Translation Key Categories

#### CLI Section (`cli.*`)
- Application descriptions and help text
- Command descriptions
- Argument and option help text
- Error messages for CLI operations

#### Export Section (`export.*`)
- Document section headers (Types, Parameters, etc.)
- Table column headers (Name, Type, Required, etc.)
- Empty state messages ("No types defined", etc.)
- Field labels (Minimum Value, Maximum Value, etc.)

#### Common Section (`common.*`)
- Frequently used terms (Yes/No, etc.)
- Generic labels and values

#### Error Section (`error.*`)
- Error messages
- Warning messages
- Status messages

## Adding a New Language

To add support for a new language:

1. **Add the language enum variant** in `src/localization/mod.rs`:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
   pub enum Language {
       // ... existing languages ...
       #[serde(rename = "pt")]
       Portuguese,
   }
   ```

2. **Update language methods** in the same file:
   ```rust
   pub fn code(&self) -> &'static str {
       match self {
           // ... existing mappings ...
           Language::Portuguese => "pt",
       }
   }

   pub fn name(&self) -> &'static str {
       match self {
           // ... existing mappings ...
           Language::Portuguese => "Portuguese",
       }
   }

   pub fn from_code(code: &str) -> Option<Self> {
       match code.to_lowercase().as_str() {
           // ... existing mappings ...
           "pt" | "pt-br" | "pt-pt" => Some(Language::Portuguese),
           _ => None,
       }
   }
   ```

3. **Create the translation file** `locales/pt.yml` using `en.yml` as a template.

4. **Update the translation loader** in `src/localization/translations.rs`:
   ```rust
   fn load_language_translations(language: Language) -> Result<HashMap<String, String>, LocalizationError> {
       let json_content = match language {
           // ... existing cases ...
           Language::Portuguese => include_str!("../locales/pt.yml"),
       };
       // ...
   }
   ```

5. **Test your translation** by running the demonstration test:
   ```bash
   cargo test localization_demo::demonstrate_translations -- --nocapture
   ```

## Translation Guidelines

### General Principles

1. **Accuracy**: Translations should convey the same meaning as the English original
2. **Consistency**: Use consistent terminology throughout the translation
3. **Context**: Consider the technical context - some terms may be better left in English
4. **Brevity**: Keep translations concise while maintaining clarity
5. **Cultural sensitivity**: Adapt to local conventions when appropriate

### Technical Terms

Some technical terms should generally remain in English or use established translations:
- "Bicep" (product name)
- "Azure" (product name)
- "JSON", "YAML", "Markdown", "AsciiDoc" (format names)
- Common programming terms that are widely used in English

### Format Strings

Some translations may include format placeholders like `{0}`, `{1}`. Ensure these are preserved:
```yaml
message_with_args: "File {0} exported to {1}"
```

### Emojis and Symbols

Emojis (✅/❌) are handled separately from text translations and should not be included in the translation strings.

## Testing Translations

### Running Tests
```bash
# Test all translations
cargo test --all-features

# Test specific localization functionality
cargo test localization

# Run with output to see translation examples
cargo test localization_demo::demonstrate_translations -- --nocapture
```

### Manual Testing
```bash
# Test CLI with different languages
./target/debug/bicep-docs --language es --help
./target/debug/bicep-docs --language fr markdown --help

# Test document generation
echo 'param name string' > test.bicep
./target/debug/bicep-docs --language de markdown test.bicep
```

## Updating Existing Translations

When updating translations:

1. **Check for new keys** - Compare with `en.yml` to ensure all keys are present
2. **Verify consistency** - Ensure terminology is consistent across the file
3. **Test the changes** - Run tests and manual verification
4. **Review formatting** - Ensure YAML is properly formatted

### Finding Missing Translations

You can identify missing translation keys by:
1. Running tests - missing keys will show as `[key.name]` in output
2. Checking the fallback behavior in the translator
3. Comparing translation files for key completeness

## Translation File Maintenance

### Adding New Translation Keys

When adding new translatable strings to the code:

1. **Add the key** to `TranslationKey` enum in `src/localization/mod.rs`
2. **Update the key() method** to return the appropriate dot-notation string
3. **Add translations** to all language files, starting with `en.yml`
4. **Update the translator** calls in the code to use the new key

### Removing Obsolete Keys

When removing unused translation keys:
1. Remove from the `TranslationKey` enum
2. Remove from all translation files
3. Ensure no code still references the old key

## Community Contributions

### Translation Review Process

1. **Submit translations** via pull request
2. **Include context** - explain any translation choices that might need clarification
3. **Test thoroughly** - ensure all functionality works with your translations
4. **Document changes** - note any significant translation decisions

### Quality Assurance

- All translations should be reviewed by native speakers when possible
- Consider regional variations (e.g., Latin American vs. European Spanish)
- Test translations in context, not just in isolation
- Verify that UI layout still works with translated text (text length variations)

## Current Implementation Status

### Completed
- ✅ Localization infrastructure with YAML-based translations
- ✅ CLI language flag (`--language`) with system locale detection
- ✅ 6 language translations (en, es, fr, de, ja, zh)
- ✅ Automatic fallback to English for missing translations
- ✅ Comprehensive test coverage

### In Progress
- 🔄 Integration of translations into export modules
- 🔄 CLI help text localization

### Planned
- 📋 Generated documentation localization
- 📋 Error message localization
- 📋 Additional language support based on community demand

## Getting Help

For questions about translations or localization:
1. Check existing translation files for examples
2. Review the localization module documentation
3. Run the test suite to understand expected behavior
4. Create an issue for questions or discussion

## File Encoding

All translation files should be saved in UTF-8 encoding to support international characters properly.