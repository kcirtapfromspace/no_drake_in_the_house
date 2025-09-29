{{/*
Expand the name of the chart.
*/}}
{{- define "kiro.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "kiro.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "kiro.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "kiro.labels" -}}
helm.sh/chart: {{ include "kiro.chart" . }}
{{ include "kiro.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "kiro.selectorLabels" -}}
app.kubernetes.io/name: {{ include "kiro.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Backend labels
*/}}
{{- define "kiro.backend.labels" -}}
{{ include "kiro.labels" . }}
app.kubernetes.io/component: backend
{{- end }}

{{/*
Backend selector labels
*/}}
{{- define "kiro.backend.selectorLabels" -}}
{{ include "kiro.selectorLabels" . }}
app.kubernetes.io/component: backend
{{- end }}

{{/*
Frontend labels
*/}}
{{- define "kiro.frontend.labels" -}}
{{ include "kiro.labels" . }}
app.kubernetes.io/component: frontend
{{- end }}

{{/*
Frontend selector labels
*/}}
{{- define "kiro.frontend.selectorLabels" -}}
{{ include "kiro.selectorLabels" . }}
app.kubernetes.io/component: frontend
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "kiro.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "kiro.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Backend image
*/}}
{{- define "kiro.backend.image" -}}
{{- if .Values.global.imageRegistry }}
{{- printf "%s/%s:%s" .Values.global.imageRegistry .Values.backend.image.repository .Values.backend.image.tag }}
{{- else }}
{{- printf "%s:%s" .Values.backend.image.repository .Values.backend.image.tag }}
{{- end }}
{{- end }}

{{/*
Frontend image
*/}}
{{- define "kiro.frontend.image" -}}
{{- if .Values.global.imageRegistry }}
{{- printf "%s/%s:%s" .Values.global.imageRegistry .Values.frontend.image.repository .Values.frontend.image.tag }}
{{- else }}
{{- printf "%s:%s" .Values.frontend.image.repository .Values.frontend.image.tag }}
{{- end }}
{{- end }}