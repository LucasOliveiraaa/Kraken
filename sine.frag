#version 430 core

in vec2 uv;
out vec4 fragColor;

// ---- camera uniforms ----
uniform mat4 invView;
uniform mat4 invProj;
uniform vec3 cameraPos;
uniform vec2 resolution;

// ---- constants ----
const uint INVALID_INDEX = 0xFFFFFFFFu;
const float EPS = 1.19e-7;
const uint EPS_CONSTANT = 10;
const float T0 = 10.0;
const float T1 = 100.0;

struct Plane {
  float height, maxH, minH;
  uint firstFuncIdx;
  uint funcCount;
};

struct QuadNode {
  uint children;
  uint firstIdx;
  uint planeCount;
  vec2 position;
  float size;
};

struct Volume {
  uint baseNodeIdx;
  vec3 minP, maxP;
  mat4 wordToVolume;
};

// ---- SSBOs ----
layout(std430, binding = 1) buffer FunctionBuffer { float amplitude[]; };
layout(std430, binding = 2) buffer FrequencyXBuffer { float frequencyX[]; };
layout(std430, binding = 3) buffer FrequencyYBuffer { float frequencyY[]; };
layout(std430, binding = 4) buffer PhaseBuffer { float phase[]; };

layout(std430, binding = 5) buffer Planes { Plane planes[]; }
planeBuffer;
layout(std430, binding = 6) buffer QuadTree { QuadNode nodes[]; }
quadtreeBuffer;
layout(std430, binding = 7) buffer Volumes { Volume volumes[]; }
volumeBuffer;

// ---- ray ----
struct Ray {
  vec3 origin, direction;
};

// ================================================================
// Function evaluation
// ================================================================
struct FunctionEvaluation {
  float height;
  vec2 grad;
};

FunctionEvaluation evalFunction(uint index, float x, float y, float T) {
  float a = amplitude[index];
  float w = frequencyX[index];
  float v = frequencyY[index];
  float p = phase[index];

  float component = w * x + v * y + p;
  float s = sin(component);
  float c = cos(component);

  float ac = a * c;
  vec2 grad = vec2(ac * w, ac * v);
  return FunctionEvaluation(a * s, grad);
}

uint evalFunctionCount(in Plane plane, float T) {
  if (T < 10.0)
    return plane.funcCount;
  if (T < 20.0)
    return plane.funcCount / 2u;
  if (T < 40.0)
    return plane.funcCount / 4u;
  if (T < 80.0)
    return plane.funcCount / 8u;

  return max(8u, plane.funcCount / 16u);
}

FunctionEvaluation evalFunctions(in Plane plane, float x, float y, float T) {
  float height = 0.0;
  vec2 grad = vec2(0.0);
  for (uint i = 0u; i < evalFunctionCount(plane, T); i++) {
    FunctionEvaluation r = evalFunction(plane.firstFuncIdx + i, x, y, T);
    height += r.height;
    grad += r.grad;
  }
  return FunctionEvaluation(height, grad);
}

// ================================================================
// ISF / Newton
// ================================================================
Ray normalizeRay(in Ray ray, vec3 origin, float size) {
  return Ray((ray.origin - origin) / size, ray.direction / size);
}

struct ISFEvaluation {
  float isf, jacobian;
  FunctionEvaluation eval;
};

ISFEvaluation evalISF(in Plane plane, Ray normRay, Ray origRay, float t,
                      float T) {
  vec2 proj = normRay.origin.xy + normRay.direction.xy * t;
  float z = origRay.origin.z + origRay.direction.z * t;
  FunctionEvaluation eval = evalFunctions(plane, proj.x, proj.y, T);
  float height = plane.height + eval.height;
  float isf = height - z;
  float jacobian = dot(eval.grad, normRay.direction.xy) - normRay.direction.z;
  return ISFEvaluation(isf, jacobian, eval);
}

struct NewtonStep {
  float t, isf;
  FunctionEvaluation eval;
};

NewtonStep newtonStep(in Plane plane, Ray normRay, Ray origRay, float t,
                      float T) {
  ISFEvaluation eval = evalISF(plane, normRay, origRay, t, T);
  return NewtonStep(t - (eval.isf / eval.jacobian), eval.isf, eval.eval);
}

// ================================================================
// Plane / domain hit
// ================================================================
struct Hit {
  Ray ray;
  vec3 point;
  vec3 normal;
  bool hit;
  float t;
};

float evalEpsilon(float T) { return max(1e-5, float(EPS_CONSTANT) * EPS * T); }

Hit hitPlane(in QuadNode domain, in Plane plane, Ray ray, float accT, float T,
             float entryT, float exitT) {
  Ray normRay = normalizeRay(ray, vec3(domain.position, 0.0), domain.size);
  float epsilon = evalEpsilon(T);
  float t = accT;
  for (uint i = 0u; i < 5u; i++) {
    NewtonStep nStep = newtonStep(plane, normRay, ray, t, T + t);
    if (abs(nStep.isf) <= epsilon) {
      vec3 normal =
          normalize(vec3(-nStep.eval.grad.x, -nStep.eval.grad.y, 1.0));

      return Hit(ray, ray.origin + ray.direction * t, normal, true, t);
    }
    t = nStep.t;
    epsilon = evalEpsilon(T + t);

    if (t < entryT || t > exitT)
      break;
  }
  return Hit(ray, vec3(0.0), vec3(0.0), false, 0.0);
}

Hit hitDomain(in QuadNode domain, Ray ray, float T, float entryT, float exitT) {
  float entryZ = ray.origin.z + ray.direction.z * entryT;
  float exitZ = ray.origin.z + ray.direction.z * exitT;
  float maxH = max(entryZ, exitZ);
  float minH = min(entryZ, exitZ);

  uint flip = uint(ray.direction.z > 0.0);
  for (uint i = 0u; i < domain.planeCount; ++i) {
    uint idx = i ^ (flip * (domain.planeCount - 1u));
    Plane p = planeBuffer.planes[domain.firstIdx + idx];

    bool above = p.minH > maxH;
    bool below = p.maxH < minH;
    bool skip = (above && flip == 0u) || (below && flip == 1u);
    bool stop = (below && flip == 0u) || (above && flip == 1u);

    if (stop)
      break;
    if (skip)
      continue;

    float tEnterPlaneZ = (ray.direction.z > 0.0)
                             ? (p.minH - ray.origin.z) / ray.direction.z
                             : (p.maxH - ray.origin.z) / ray.direction.z;

    float accT_plane = max(entryT, tEnterPlaneZ);
    Hit hit = hitPlane(domain, p, ray, accT_plane, T, entryT, exitT);
    if (hit.hit)
      return hit;
  }

  return Hit(ray, vec3(0.0), vec3(0.0), false, 0.0);
}

// ================================================================
// Quad intersection + volume traversal
// ================================================================
struct StackItem {
  uint nodeIdx;
  float tEntry, tExit;
};

bool intersectQuadXY(Ray ray, vec2 minP, vec2 maxP, float tMin, float tMax,
                     out float entryT, out float exitT) {
  vec2 invDir = 1.0 / ray.direction.xy;
  vec2 t0 = (minP - ray.origin.xy) * invDir;
  vec2 t1 = (maxP - ray.origin.xy) * invDir;
  vec2 tmin2 = min(t0, t1);
  vec2 tmax2 = max(t0, t1);
  entryT = max(max(tmin2.x, tmin2.y), tMin);
  exitT = min(min(tmax2.x, tmax2.y), tMax);
  return entryT <= exitT;
}

Hit hitVolume(uint volumeIdx, Ray worldRay, float T, float tMin, float tMax) {
  Volume vol = volumeBuffer.volumes[volumeIdx];

  Ray ray;
  ray.origin = (vol.wordToVolume * vec4(worldRay.origin, 1.0)).xyz;
  ray.direction =
      normalize((vol.wordToVolume * vec4(worldRay.direction, 0.0)).xyz);

  StackItem stack[32];
  int sp = 0;
  stack[sp++] = StackItem(vol.baseNodeIdx, tMin, tMax);

  Hit bestHit;
  bestHit.hit = false;

  while (sp > 0) {
    StackItem item = stack[--sp];
    QuadNode node = quadtreeBuffer.nodes[item.nodeIdx];

    float entryT, exitT;
    if (!intersectQuadXY(ray, node.position, node.position + vec2(node.size),
                         item.tEntry, item.tExit, entryT, exitT))
      continue;

    if (node.children == 0u) {
      Hit h = hitDomain(node, ray, T, entryT, exitT);
      if (h.hit)
        return h;
    } else {
      uint base = node.firstIdx;
      StackItem childItems[4];
      int childCount = 0;

      for (int i = 0; i < 4; i++) {
        QuadNode c = quadtreeBuffer.nodes[base + i];
        float cEntry, cExit;
        if (intersectQuadXY(ray, c.position, c.position + vec2(c.size), entryT,
                            exitT, cEntry, cExit))
          childItems[childCount++] = StackItem(uint(base + i), cEntry, cExit);
      }

      // Bubble sort by tEntry (4 items max)
      for (int i = 0; i < childCount - 1; i++)
        for (int j = i + 1; j < childCount; j++)
          if (childItems[j].tEntry < childItems[i].tEntry) {
            StackItem tmp = childItems[i];
            childItems[i] = childItems[j];
            childItems[j] = tmp;
          }

      // Push far → near so near is popped first
      for (int i = childCount - 1; i >= 0; i--)
        stack[sp++] = childItems[i];
    }
  }

  return bestHit;
}

const vec3 lightColor = vec3(1.0, 1.0, 1.0);
const vec3 lightPos = vec3(0.0, 0.0, 10.0);
const float lightRadius = 1.0;

bool hitLight(Ray ray) {
  vec3 oc = ray.origin - lightPos;

  float a = dot(ray.direction, ray.direction);
  float b = 2.0 * dot(oc, ray.direction);
  float c = dot(oc, oc) - lightRadius * lightRadius;

  float d = b * b - 4.0 * a * c;
  if (d < 0.0)
    return false;

  float s = sqrt(d);

  float t0 = (-b - s) / (2.0 * a);
  float t1 = (-b + s) / (2.0 * a);

  return t0 > 0.0 || t1 > 0.0;
}

// ================================================================
// Main
// ================================================================
void main() {
  vec2 ndc = uv * 2.0 - 1.0;
  vec4 clip = vec4(ndc, -1.0, 1.0);
  vec4 view = invProj * clip;
  view.xyz /= view.w;

  float totalT = 0.0;

  Ray ray;
  ray.origin = cameraPos;
  ray.direction = normalize((invView * vec4(view.xyz, 0.0)).xyz);

  vec3 color = vec3(0.0);
  vec3 throughput = vec3(1.0);

  for (uint i = 0u; i < 4u; i++) {
    Hit hit = hitVolume(0u, ray, totalT, 0.001, 1e6);

    if (!hit.hit) {
      if (hitLight(ray))
        color += throughput * lightColor;
      break;
    }

    vec3 L = normalize(lightPos - hit.point);
    float diffuse = max(dot(hit.normal, L), 0.0);
    color += throughput * diffuse * lightColor;

    vec3 n = hit.normal;

    if (dot(n, ray.direction) > 0.0)
      n = -n;

    ray.origin = hit.point + n * 1e-3;
    ray.direction = reflect(ray.direction, n);

    totalT += hit.t;
    throughput *= 0.9;
  }

  fragColor = vec4(color.xyz, 1.0);
}