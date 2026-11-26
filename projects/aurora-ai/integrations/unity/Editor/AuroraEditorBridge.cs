#if UNITY_EDITOR
using System;
using System.IO;
using System.Linq;
using UnityEditor;
using UnityEditor.Build.Reporting;
using UnityEngine;

namespace Aurora
{
    /// <summary>
    /// Optional, local-only Unity Editor actions for Aurora AI.
    /// The file is installed through Aurora's existing preview + explicit approval flow.
    /// It never starts a game or sends project data over a network.
    /// </summary>
    public static class EditorBridge
    {
        private static string ProjectRoot => Directory.GetParent(Application.dataPath).FullName;

        [MenuItem("Tools/Aurora AI/Refresh Asset Database")]
        public static void RefreshAssets()
        {
            AssetDatabase.Refresh(ImportAssetOptions.ForceSynchronousImport);
            Debug.Log("[Aurora AI] Asset database refresh completed.");
            ExitBatch(0);
        }

        [MenuItem("Tools/Aurora AI/Audit Build Scenes")]
        public static void AuditBuildScenes()
        {
            var scenes = EditorBuildSettings.scenes ?? Array.Empty<EditorBuildSettingsScene>();
            var enabled = scenes.Where(scene => scene != null && scene.enabled).ToArray();
            if (enabled.Length == 0)
            {
                Debug.LogError("[Aurora AI] No enabled scenes are configured in Build Settings.");
                ExitBatch(2);
                return;
            }

            var errors = 0;
            foreach (var scene in enabled)
            {
                string fullPath;
                try { fullPath = ResolveProjectPath(scene.path); }
                catch (Exception exception)
                {
                    Debug.LogError("[Aurora AI] Unsafe build scene path: " + scene.path + " · " + exception.Message);
                    errors++;
                    continue;
                }
                if (!File.Exists(fullPath))
                {
                    Debug.LogError("[Aurora AI] Enabled build scene is missing: " + scene.path);
                    errors++;
                }
                else Debug.Log("[Aurora AI] Build scene OK: " + scene.path);
            }

            Debug.Log($"[Aurora AI] Scene audit complete. Enabled: {enabled.Length}; errors: {errors}.");
            ExitBatch(errors == 0 ? 0 : 2);
        }

        [MenuItem("Tools/Aurora AI/Build Windows Player")]
        public static void BuildWindowsPlayer()
        {
            var scenes = (EditorBuildSettings.scenes ?? Array.Empty<EditorBuildSettingsScene>())
                .Where(scene => scene != null && scene.enabled)
                .Select(scene => scene.path)
                .ToArray();
            if (scenes.Length == 0 || scenes.Any(scene =>
            {
                try { return !File.Exists(ResolveProjectPath(scene)); }
                catch { return true; }
            }))
            {
                Debug.LogError("[Aurora AI] Build stopped: enabled build scenes are missing or not configured.");
                ExitBatch(2);
                return;
            }

            var outputDirectory = Path.GetFullPath(Path.Combine(ProjectRoot, "Build", "Aurora", "Windows"));
            var projectPrefix = ProjectRoot.TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar;
            if (!outputDirectory.StartsWith(projectPrefix, StringComparison.OrdinalIgnoreCase))
            {
                Debug.LogError("[Aurora AI] Build output path is outside the project.");
                ExitBatch(2);
                return;
            }

            Directory.CreateDirectory(outputDirectory);
            var report = BuildPipeline.BuildPlayer(new BuildPlayerOptions
            {
                scenes = scenes,
                locationPathName = Path.Combine(outputDirectory, "AuroraGame.exe"),
                target = BuildTarget.StandaloneWindows64,
                options = BuildOptions.None
            });

            if (report.summary.result == BuildResult.Succeeded)
            {
                Debug.Log($"[Aurora AI] Windows build succeeded: {report.summary.totalSize} bytes; output: {report.summary.outputPath}");
                ExitBatch(0);
            }
            else
            {
                Debug.LogError("[Aurora AI] Windows build failed: " + report.summary.result);
                ExitBatch(2);
            }
        }

        private static string ResolveProjectPath(string relativePath)
        {
            var root = Path.GetFullPath(ProjectRoot).TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar;
            var fullPath = Path.GetFullPath(Path.Combine(root, relativePath));
            if (!fullPath.StartsWith(root, StringComparison.OrdinalIgnoreCase))
                throw new InvalidOperationException("Path is outside the Unity project.");
            return fullPath;
        }

        private static void ExitBatch(int code)
        {
            if (Application.isBatchMode) EditorApplication.Exit(code);
        }
    }
}
#endif
