import { useEffect, useState } from "react";
import { motion } from "framer-motion";

// We'll use a simple regex-based markdown parser for now
// In a production app, you'd want to use a library like marked or remark
const MarkdownRenderer = ({ content }: { content: string }) => {
  const [html, setHtml] = useState("");
  const [frontMatter, setFrontMatter] = useState<Record<string, any> | null>(
    null,
  );

  useEffect(() => {
    // Simple markdown parser implementation
    const parseMarkdown = (md: string) => {
      // Extract YAML frontmatter
      const frontMatterMatch = md.match(/^---([\s\S]*?)---\n/);

      if (frontMatterMatch && frontMatterMatch[1]) {
        // Basic YAML parsing
        const yamlContent = frontMatterMatch[1];
        const frontMatterObj: Record<string, any> = {};

        // Process line by line
        yamlContent.split("\n").forEach((line) => {
          const trimmedLine = line.trim();
          if (trimmedLine && !trimmedLine.startsWith("#")) {
            const [key, ...valueParts] = trimmedLine.split(":");
            if (key && valueParts.length) {
              const value = valueParts.join(":").trim();

              // Handle arrays (basic support)
              if (value.startsWith("[") && value.endsWith("]")) {
                frontMatterObj[key.trim()] = value
                  .substring(1, value.length - 1)
                  .split(",")
                  .map((item) => item.trim());
              } else {
                frontMatterObj[key.trim()] = value;
              }
            }
          }
        });

        setFrontMatter(frontMatterObj);
      } else {
        setFrontMatter(null);
      }

      // Remove YAML frontmatter and clean up content
      let processedMd = md.replace(/^---[\s\S]*?---\n/, "");

      // Remove "Back to Index" links that might be in the content
      processedMd = processedMd.replace(/^Back to Index\s*$/gm, "");

      // Process YAML-like syntax in the content body (e.g., "---" section markers)
      processedMd = processedMd.replace(/^---\s*$/gm, "");

      // Remove duplicate note titles
      if (frontMatterMatch && frontMatterMatch[1]) {
        const titleMatch = frontMatterMatch[1].match(/title:\s*(.*?)$/m);
        if (titleMatch && titleMatch[1]) {
          const titleText = titleMatch[1].trim();
          // Remove any markdown headings with the same title
          processedMd = processedMd.replace(new RegExp(`^# ${titleText}\\s*$`, "gm"), "");
        }
      }
      

      // Headers
      processedMd = processedMd.replace(/^# (.*$)/gim, "<h1>$1</h1>");
      processedMd = processedMd.replace(/^## (.*$)/gim, "<h2>$1</h2>");
      processedMd = processedMd.replace(/^### (.*$)/gim, "<h3>$1</h3>");
      processedMd = processedMd.replace(/^#### (.*$)/gim, "<h4>$1</h4>");

      // Bold and italic
      processedMd = processedMd.replace(
        /\*\*(.*?)\*\*/gim,
        "<strong>$1</strong>",
      );
      processedMd = processedMd.replace(/\*(.*?)\*/gim, "<em>$1</em>");

      // Lists - improve shopping list handling
      processedMd = processedMd.replace(
        /^\s*\n\* (.*)/gim,
        "<ul>\n<li>$1</li>",
      );
      // Improved handling for bullet points
      processedMd = processedMd.replace(/^\* (.*)/gim, "<li class=\"ml-6 list-disc\">$1</li>");
      processedMd = processedMd.replace(
        /^\s*\n\- (.*)/gim,
        "<ul>\n<li>$1</li>",
      );
      processedMd = processedMd.replace(/^\- (.*)/gim, "<li class=\"ml-6 list-disc\">$1</li>");

      // Improved numbered lists
      processedMd = processedMd.replace(
        /^([0-9]+)\. (.*)/gim,
        '<ol start="$1">\n<li class="ml-6 list-decimal">$2</li>',
      );

      // Links
      processedMd = processedMd.replace(
        /\[([^\[]+)\]\(([^\)]+)\)/gim,
        '<a href="$2" target="_blank" rel="noopener noreferrer" class="text-primary-500 hover:underline">$1</a>',
      );

      // Wiki-style links
      processedMd = processedMd.replace(
        /\[\[(.*?)\]\]/gim,
        '<a href="/notes/$1" class="text-primary-500 hover:underline">$1</a>',
      );

      // Code blocks
      processedMd = processedMd.replace(
        /```([^`]*?)```/gim,
        "<pre><code>$1</code></pre>",
      );

      // Inline code
      processedMd = processedMd.replace(/`([^`]*?)`/gim, "<code>$1</code>");

      // Blockquotes
      processedMd = processedMd.replace(
        /^\> (.*$)/gim,
        "<blockquote>$1</blockquote>",
      );

      // Paragraphs
      processedMd = processedMd.replace(/^\s*(\n)?(.+)/gim, function (match) {
        const trimmedMatch = match.trim();
        if (/^<\/?(ul|ol|li|h|p|bl|code)/i.test(trimmedMatch)) {
          return match;
        }
        return "<p>" + trimmedMatch + "</p>";
      });

      // Special handling for potential shopping list items or checklists
      processedMd = processedMd.replace(/^\* \[ \] (.*)/gm, '<div class="flex items-center gap-2 py-1"><input type="checkbox" class="form-checkbox h-5 w-5 text-primary-500" disabled /><span class="text-gray-700 dark:text-gray-300">$1</span></div>');
      processedMd = processedMd.replace(/^\* \[x\] (.*)/gm, '<div class="flex items-center gap-2 py-1"><input type="checkbox" class="form-checkbox h-5 w-5 text-primary-500" disabled checked /><span class="text-gray-700 dark:text-gray-300 line-through">$1</span></div>');

      // Improved handling for regular list items (like "* dog food")
      processedMd = processedMd.replace(/^\* ([^[].*)/gm, '<div class="flex items-center gap-2 py-1 ml-2"><span class="h-2 w-2 rounded-full bg-primary-500 dark:bg-primary-400 inline-block mr-2"></span><span class="text-gray-700 dark:text-gray-300">$1</span></div>');

      // Line breaks
      processedMd = processedMd.replace(/\n/gim, "<br>");

      return processedMd.trim();
    };

    setHtml(parseMarkdown(content));
  }, [content]);

  // Extract the actual content sections for proper parsing
  const hasYamlDelimiter = content.match(/^---[\s\S]*?---\n/);
  const actualContent = content.replace(/^---[\s\S]*?---\n/, '');
  const hasHeader = actualContent.match(/^#\s+(.+?)$/m);

  // Check if content is mostly empty (just placeholders)
  const isEmptyContent = html.trim() === "" ||
    (html.includes("Write your note here") && html.length < 50);

  // Check if content contains "Back to Index" or placeholder text
  const cleanedHtml = html
    .replace(/<p>Back to Index<\/p>/g, '') // Remove "Back to Index" paragraphs
    .replace(/<p>Write your note here\.<\/p>/g, ''); // Remove placeholder text

  return (
    <div>
      {/* Render frontmatter if available */}
      {frontMatter && Object.keys(frontMatter).length > 0 && (
        <motion.div
          className="bg-gray-50 dark:bg-gray-800/50 rounded-lg p-3 border border-gray-200 dark:border-gray-700 mb-3"
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          transition={{ duration: 0.4 }}
        >
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            {Object.entries(frontMatter).map(([key, value], index) => (
              <motion.div
                key={key}
                className="flex gap-2"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.3 + index * 0.05 }}
              >
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  {key}:
                </span>
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  {Array.isArray(value)
                    ? value.join(", ")
                    : typeof value === "string"
                      ? value
                      : JSON.stringify(value)}
                </span>
              </motion.div>
            ))}
          </div>
        </motion.div>
      )}

      {/* Render markdown content - only if it has meaningful content beyond placeholders */}
      {cleanedHtml.trim() !== "" && (
        <motion.div
          className="prose dark:prose-invert max-w-none"
          dangerouslySetInnerHTML={{ __html: cleanedHtml }}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
        />
      )}
    </div>
  );
};

export default MarkdownRenderer;
