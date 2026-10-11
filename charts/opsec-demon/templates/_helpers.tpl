{{- define "opsec-demon.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- define "opsec-demon.fullname" -}}
{{- default (printf "%s-%s" .Release.Name (include "opsec-demon.name" .)) .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- define "opsec-demon.selectorLabels" -}}
app.kubernetes.io/name: {{ include "opsec-demon.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}
{{- define "opsec-demon.labels" -}}
{{ include "opsec-demon.selectorLabels" . }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}
